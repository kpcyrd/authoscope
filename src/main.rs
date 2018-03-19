#![warn(unused_extern_crates)]
extern crate hlua;
extern crate pbr;
extern crate threadpool;
extern crate colored;
extern crate time;
extern crate humantime;
extern crate atty;
extern crate rand;
extern crate getch;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate structopt;

#[cfg(not(windows))]
extern crate termios;

extern crate reqwest;
extern crate mysql;
extern crate ldap3;

mod args;
mod ctx;
mod keyboard;
mod pb;
mod runtime;
mod scheduler;

use pb::ProgressBar;
use error_chain::ChainedError;
use colored::*;
use scheduler::{Scheduler, Attempt, Msg};
use keyboard::{Keyboard, Key};
use std::thread;
use std::fs::{self, File};
use std::sync::Arc;
use std::time::Instant;
use std::io::{self, BufReader};
use std::io::prelude::*;

mod errors {
    use std;
    use hlua;

    error_chain! {
        foreign_links {
            Io(std::io::Error);
            Lua(hlua::LuaError);
        }
    }
}
use errors::{Result, ResultExt};

fn load_list(path: &str) -> Result<Vec<Arc<String>>> {
    let f = File::open(path)?;
    let file = BufReader::new(&f);
    let lines: io::Result<_> = file.lines()
            .map(|x| x.map(|x| Arc::new(x)))
            .collect();
    Ok(lines?)
}

fn load_scripts(paths: Vec<String>) -> Result<Vec<Arc<ctx::Script>>> {
    let mut scripts = Vec::new();

    for path in paths {
        let meta = fs::metadata(&path)?;

        if meta.is_dir() {
            for path in fs::read_dir(path)? {
                let path = path?.path();
                let path = path.to_str().unwrap();
                let script = Arc::new(ctx::Script::load(path)?);
                scripts.push(script);
            }
        } else {
            let script = Arc::new(ctx::Script::load(&path)?);
            scripts.push(script);
        }
    }

    Ok(scripts)
}

macro_rules! infof {
    ($arg1:tt, $fmt:expr, $($arg:tt)*) => (
        $arg1.bold().to_string() + " " + &(format!($fmt, $($arg)*).dimmed().to_string())
    );
}

macro_rules! info {
    ($arg1:tt, $fmt:expr, $($arg:tt)*) => (
        println!("{}", infof!($arg1, $fmt, $($arg)*));
    );
}

fn setup_dictionary_attack(pool: &mut Scheduler, args: args::Dict) -> Result<(usize, Instant)> {
    let users = load_list(&args.users).chain_err(|| "failed to load users")?;
    info!("[+]", "loaded {} users", users.len());
    let passwords = load_list(&args.passwords).chain_err(|| "failed to load passwords")?;
    info!("[+]", "loaded {} passwords", passwords.len());
    let scripts = load_scripts(args.scripts).chain_err(|| "failed to load scripts")?;
    info!("[+]", "loaded {} scripts", scripts.len());

    let attempts = users.len() * passwords.len() * scripts.len();
    info!("[*]", "submitting {} jobs to threadpool with {} workers", attempts, pool.max_count());
    let start = Instant::now();

    for user in &users {
        for password in &passwords {
            for script in &scripts {
                let attempt = Attempt::new(user, password, script);
                pool.run(attempt);
            }
        }
    }

    Ok((attempts, start))
}

fn setup_credential_confirmation(pool: &mut Scheduler, args: args::Creds) -> Result<(usize, Instant)> {
    let creds = load_list(&args.creds)
                    .chain_err(|| "failed to load creds")?
                    .into_iter()
                    .map(|x| {
                        if let Some(idx) = x.find(":") {
                            let (user, password) = x.split_at(idx);
                            Ok((Arc::new(user.to_owned()), Arc::new(password[1..].to_owned())))
                        } else {
                            Err(format!("invalid list format: {:?}", x).into())
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
    info!("[+]", "loaded {} credentials", creds.len());
    let scripts = load_scripts(args.scripts).chain_err(|| "failed to load scripts")?;
    info!("[+]", "loaded {} scripts", scripts.len());

    let attempts = creds.len() * scripts.len();
    info!("[*]", "submitting {} jobs to threadpool with {} workers", attempts, pool.max_count());
    let start = Instant::now();

    for (user, password) in creds {
        for script in &scripts {
            let attempt = Attempt::new(&user, &password, script);
            pool.run(attempt);
        }
    }

    Ok((attempts, start))
}

fn run() -> Result<()> {
    let args = args::parse();

    if atty::isnt(atty::Stream::Stdout) {
        colored::control::SHOULD_COLORIZE.set_override(false);
    }

    let mut pool = Scheduler::new(args.workers);

    let (attempts, start) = match args.subcommand {
        args::SubCommand::Dict(dict) => setup_dictionary_attack(&mut pool, dict)?,
        args::SubCommand::Creds(creds) => setup_credential_confirmation(&mut pool, creds)?,
    };

    let tx = pool.tx();
    thread::spawn(move || {
        let kb = Keyboard::new();
        loop {
            let key = kb.get();
            tx.send(Msg::Key(key)).expect("failed to send key");
        }
    });

    let mut pb = ProgressBar::new(attempts as u64);
    pb.tick();

    let mut valid = 0;
    let mut retries = 0;
    let mut expired = 0;
    while pool.has_work() {
        match pool.recv() {
            Msg::Key(key) => {
                match key {
                    Key::P => {
                        pb.writeln(format!("{} {}", "[*]".bold(), "pausing threads".dimmed()));
                        pool.pause();
                    },
                    Key::R => {
                        pb.writeln(format!("{} {}", "[*]".bold(), "resuming threads".dimmed()));
                        pool.resume();
                    },
                    Key::Plus => {
                        println!("plus!");
                    },
                    Key::Minus => {
                        println!("minus!");
                    },
                }
                pb.tick();
            },
            Msg::Attempt(mut attempt, result) => {
                match result {
                    Ok(is_valid) => {
                        if is_valid {
                            pb.writeln(format!("{} {}({}) => {:?}:{:?}", "[+]".bold(), "valid".green(),
                                attempt.script.descr().yellow(), attempt.user, attempt.password));
                            valid += 1;
                        }
                        pb.inc();
                    },
                    Err(err) => {
                        pb.writeln(format!("{} {}({}, {}): {:?}", "[!]".bold(), "error".red(), attempt.script.descr().yellow(), format!("{:?}:{:?}", attempt.user, attempt.password).dimmed(), err));

                        if attempt.ttl > 0 {
                            // we have retries left
                            retries += 1;
                            attempt.ttl -= 1;
                            pool.run(attempt);
                            pb.tick();
                        } else {
                            // giving up
                            expired += 1;
                            pb.inc();
                        }
                    }
                };
            },
        }
    }

    let elapsed = start.elapsed();
    let average = elapsed / attempts as u32;
    pb.finish_replace(infof!("[+]", "found {} valid credentials with {} attempts and {} retries after {} and on average {} per attempt. {} attempts expired.\n",
            valid, attempts, retries,
            humantime::format_duration(elapsed),
            humantime::format_duration(average),
            expired,
    ));

    Keyboard::reset();

    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        eprintln!("{}", e.display_chain());
        std::process::exit(1);
    }
}
