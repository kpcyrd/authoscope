#![warn(unused_extern_crates)]
extern crate badtouch;
extern crate env_logger;
extern crate colored;
extern crate humantime;
extern crate atty;
extern crate error_chain;
#[macro_use] extern crate log;

use badtouch::args;
use badtouch::ctx::Script;
use badtouch::fsck;
use badtouch::utils;
use badtouch::config::Config;
use badtouch::pb::ProgressBar;
use badtouch::scheduler::{Scheduler, Attempt, Creds, Msg};
use badtouch::keyboard::{Keyboard, Key};
use badtouch::ulimit::{Resource, getrlimit, setrlimit};

use error_chain::ChainedError;
use colored::*;
use std::thread;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;
use std::io::prelude::*;
use badtouch::errors::{Result, ResultExt};


enum Report {
    Some(File),
    None
}

impl Report {
    pub fn open(path: Option<String>) -> Result<Report> {
        match path {
            Some(path) => Ok(Report::Some(File::create(path)?)),
            None => Ok(Report::None),
        }
    }

    pub fn write_creds(&mut self, user: &str, password: &str, script: &str) -> Result<()> {
        if let Report::Some(ref mut f) = *self {
            writeln!(f, "{}:{}:{}", script, user, password)?;
        }
        Ok(())
    }

    pub fn write_enum(&mut self, user: &str, script: &str) -> Result<()> {
        if let Report::Some(ref mut f) = *self {
            writeln!(f, "{}:{}", script, user)?;
        }
        Ok(())
    }
}

macro_rules! tinfof {
    ($arg1:tt, $fmt:expr, $($arg:tt)*) => (
        $arg1.bold().to_string() + " " + &(format!($fmt, $($arg)*).dimmed().to_string())
    );
}

macro_rules! tinfo {
    ($arg1:tt, $fmt:expr, $($arg:tt)*) => (
        println!("{}", tinfof!($arg1, $fmt, $($arg)*));
    );
}

fn setup_dictionary_attack(pool: &mut Scheduler, args: args::Dict, config: &Arc<Config>) -> Result<usize> {
    let users = utils::load_list(&args.users).chain_err(|| "failed to load users")?;
    tinfo!("[+]", "loaded {} users", users.len());
    let passwords = utils::load_list(&args.passwords).chain_err(|| "failed to load passwords")?;
    tinfo!("[+]", "loaded {} passwords", passwords.len());
    let scripts = utils::load_scripts(args.scripts, &config).chain_err(|| "failed to load scripts")?;
    tinfo!("[+]", "loaded {} scripts", scripts.len());

    let attempts = users.len() * passwords.len() * scripts.len();
    tinfo!("[*]", "submitting {} jobs to threadpool with {} workers", attempts, pool.max_count());

    for user in &users {
        for password in &passwords {
            for script in &scripts {
                let attempt = Attempt::new(user, password, script);
                pool.run(attempt);
            }
        }
    }

    Ok(attempts)
}

fn setup_credential_confirmation(pool: &mut Scheduler, args: args::Creds, config: &Arc<Config>) -> Result<usize> {
    let creds = utils::load_creds(&args.creds)?;
    tinfo!("[+]", "loaded {} credentials", creds.len());
    let scripts = utils::load_scripts(args.scripts, &config).chain_err(|| "failed to load scripts")?;
    tinfo!("[+]", "loaded {} scripts", scripts.len());

    let attempts = creds.len() * scripts.len();
    tinfo!("[*]", "submitting {} jobs to threadpool with {} workers", attempts, pool.max_count());

    for cred in creds {
        // TODO: optimization if we only have once script
        for script in &scripts {
            let attempt = Attempt::bytes(&cred, script);
            pool.run(attempt);
        }
    }

    Ok(attempts)
}

fn setup_enum_attack(pool: &mut Scheduler, args: args::Enum, config: &Arc<Config>) -> Result<usize> {
    let users = utils::load_list(&args.users).chain_err(|| "failed to load users")?;
    tinfo!("[+]", "loaded {} users", users.len());
    let scripts = utils::load_scripts(args.scripts, &config).chain_err(|| "failed to load scripts")?;
    tinfo!("[+]", "loaded {} scripts", scripts.len());

    let attempts = users.len() * scripts.len();
    tinfo!("[*]", "submitting {} jobs to threadpool with {} workers", attempts, pool.max_count());

    for user in &users {
        for script in &scripts {
            let attempt = Attempt::enumerate(user, script);
            pool.run(attempt);
        }
    }

    Ok(attempts)
}

fn run_oneshot(oneshot: args::Oneshot, config: Arc<Config>) -> Result<()> {
    let script = Script::load(&oneshot.script, config)?;
    let user = oneshot.user;

    let valid = match oneshot.password {
        Some(ref password) => script.run_creds(&user, &password)?,
        None => script.run_enum(&user)?,
    };

    if valid {
        match oneshot.password {
            Some(ref password) => println!("{}", format_valid_creds(script.descr(), &user, &password)),
            None => println!("{}", format_valid_enum(script.descr(), &user)),
        }
    } else if oneshot.exitcode {
        std::process::exit(2);
    }

    Ok(())
}

fn format_valid_creds(script: &str, user: &str, password: &str) -> String {
    format!("{} {}({}) => {:?}:{:?}", "[+]".bold(), "valid".green(),
        script.yellow(), user, password)
}

fn format_valid_enum(script: &str, user: &str) -> String {
    format!("{} {}({}) => {:?}", "[+]".bold(), "valid".green(),
        script.yellow(), user)
}

fn set_nofile(config: &Config) -> Result<()> {
    let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE)?;
    debug!("soft_limit={:?}, hard_limit={:?}", soft_limit, hard_limit);

    let hard_limit = config.runtime.rlimit_nofile.or(hard_limit);
    info!("setting soft_limit to {:?}", hard_limit);
    setrlimit(Resource::RLIMIT_NOFILE, hard_limit, hard_limit)?;

    let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE)?;
    debug!("soft_limit={:?}, hard_limit={:?}", soft_limit, hard_limit);

    Ok(())
}

fn run() -> Result<()> {
    let args = args::parse();

    let env = env_logger::Env::default();
    let env = match args.verbose {
        0 => env,
        1 => env.filter_or("RUST_LOG", "info"),
        _ => env.filter_or("RUST_LOG", "debug"),
    };
    env_logger::init_from_env(env);

    if atty::isnt(atty::Stream::Stdout) {
        colored::control::SHOULD_COLORIZE.set_override(false);
    }

    let config = Arc::new(Config::load()?);
    #[cfg(target_os="linux")]
    set_nofile(&config)
        .chain_err(|| "failed to set RLIMIT_NOFILE")?;

    let mut pool = Scheduler::new(args.workers);
    let mut report = Report::open(args.output)?;

    let attempts = match args.subcommand {
        args::SubCommand::Dict(dict) => setup_dictionary_attack(&mut pool, dict, &config)?,
        args::SubCommand::Creds(creds) => setup_credential_confirmation(&mut pool, creds, &config)?,
        args::SubCommand::Enum(enumerate) => setup_enum_attack(&mut pool, enumerate, &config)?,
        args::SubCommand::Oneshot(oneshot) => return run_oneshot(oneshot, config),
        args::SubCommand::Fsck(fsck) => return fsck::run_fsck(&fsck),
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
    pb.print_help();
    pb.tick();

    pool.resume();
    let start = Instant::now();

    let mut valid = 0;
    let mut retries = 0;
    let mut expired = 0;
    while pool.has_work() {
        match pool.recv() {
            Msg::Key(key) => {
                match key {
                    Key::H => pb.print_help(),
                    Key::P => {
                        pb.writeln(format!("{} {}", "[*]".bold(), "pausing threads".dimmed()));
                        pool.pause();
                    },
                    Key::R => {
                        pb.writeln(format!("{} {}", "[*]".bold(), "resuming threads".dimmed()));
                        pool.resume();
                    },
                    Key::Plus => {
                        let num = pool.incr();
                        pb.writeln(format!("{} {}", "[*]".bold(), format!("increased to {} threads", num).dimmed()));
                    },
                    Key::Minus => {
                        let num = pool.decr();
                        pb.writeln(format!("{} {}", "[*]".bold(), format!("decreased to {} threads", num).dimmed()));
                    },
                }
                pb.tick();
            },
            Msg::Attempt(mut attempt, result) => {
                match result {
                    Ok(is_valid) => {
                        if is_valid {
                            match attempt.creds {
                                Creds::Enum(_) => {
                                    let user = attempt.user();
                                    let script = attempt.script.descr();

                                    pb.writeln(format_valid_enum(script, user));
                                    report.write_enum(user, script)?;
                                },
                                _ => {
                                    let user = attempt.user();
                                    let password = attempt.password();
                                    let script = attempt.script.descr();

                                    pb.writeln(format_valid_creds(script, user, password));
                                    report.write_creds(user, password, script)?;
                                },
                            };
                            valid += 1;
                        }
                        pb.inc();
                    },
                    Err(err) => {
                        pb.writeln(format!("{} {}({}, {}): {:?}", "[!]".bold(), "error".red(), attempt.script.descr().yellow(), format!("{:?}:{:?}", attempt.user(), attempt.password()).dimmed(), err));

                        if attempt.ttl > 0 {
                            // we have retries left
                            retries += 1;
                            attempt.ttl -= 1;
                            pool.run(*attempt);
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
    pb.finish_replace(tinfof!("[+]", "found {} valid credentials with {} attempts and {} retries after {} and on average {} per attempt. {} attempts expired.\n",
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
        eprint!("{}", e.display_chain());
        std::process::exit(1);
    }
}
