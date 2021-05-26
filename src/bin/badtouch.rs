use authoscope::args;
use authoscope::config::Config;
use authoscope::ctx::Script;
use authoscope::errors::*;
use authoscope::fsck;
use authoscope::keyboard::{Keyboard, Key};
use authoscope::pb::ProgressBar;
use authoscope::scheduler::{self, Scheduler, Attempt, Msg};
use authoscope::utils;
use colored::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use structopt::StructOpt;
use structopt::clap::{AppSettings, Shell};

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Verbose output
    #[structopt(short="v", long="verbose",
                global=true, parse(from_occurrences))]
    pub verbose: u8,
    /// Concurrent workers
    #[structopt(short = "n", long = "workers", default_value = "16")]
    pub workers: usize,
    /// Write results to file
    #[structopt(short = "o", long = "output")]
    pub output: Option<String>,
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Dictionary attack
    #[structopt(name="dict")]
    Dict(Dict),
    /// Credential confirmation attack
    #[structopt(name="creds")]
    Creds(Creds),
    /// Enumerate users
    #[structopt(name="enum")]
    Enum(Enum),
    /// Test a single username-password combination
    #[structopt(name="oneshot")]
    Oneshot(Oneshot),
    /// Verify and fix encoding of a list
    #[structopt(name="fsck")]
    Fsck(Fsck),
    Completions(Completions),
}

#[derive(Debug, StructOpt)]
pub struct Dict {
    /// Username list path
    pub users: String,
    /// Password list path
    pub passwords: String,
    /// Scripts to run
    #[structopt(required=true)]
    pub scripts: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct Creds {
    /// Credential list path
    pub creds: String,
    /// Scripts to run
    #[structopt(required=true)]
    pub scripts: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct Enum {
    /// Username list path
    pub users: String,
    /// Scripts to run
    #[structopt(required=true)]
    pub scripts: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct Oneshot {
    /// Script to run
    pub script: String,
    /// Username to test
    pub user: String,
    /// Password to test
    pub password: Option<String>,
    /// Set the exitcode to 2 if the credentials are invalid
    #[structopt(short = "x", long = "exitcode")]
    pub exitcode: bool,
}

#[derive(Debug, StructOpt)]
pub struct Fsck {
    /// Do not show invalid lines
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,
    /// Do not show valid lines
    #[structopt(short = "s", long = "silent")]
    pub silent: bool,
    /// Require one colon per line
    #[structopt(short = "c", long = "colon")]
    pub require_colon: bool,
    /// Files to read
    pub paths: Vec<String>,
}

/// Generate shell completions
#[derive(Debug, StructOpt)]
pub struct Completions {
    #[structopt(possible_values=&Shell::variants())]
    pub shell: Shell,
}

impl Completions {
    pub fn gen(&self) -> Result<()> {
        Args::clap().gen_completions_to("authoscope", self.shell, &mut stdout());
        Ok(())
    }
}
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

fn setup_dictionary_attack(pool: &mut Scheduler, args: Dict, config: &Arc<Config>) -> Result<usize> {
    let users = utils::load_list(&args.users)
        .context("Failed to load users")?;
    tinfo!("[+]", "loaded {} users", users.len());
    let passwords = utils::load_list(&args.passwords)
        .context("Failed to load passwords")?;
    tinfo!("[+]", "loaded {} passwords", passwords.len());
    let scripts = utils::load_scripts(args.scripts, &config)
        .context("Failed to load scripts")?;
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

fn setup_credential_confirmation(pool: &mut Scheduler, args: Creds, config: &Arc<Config>) -> Result<usize> {
    let creds = utils::load_combolist(&args.creds)?;
    tinfo!("[+]", "loaded {} credentials", creds.len());
    let scripts = utils::load_scripts(args.scripts, &config)
        .context("Failed to load scripts")?;
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

fn setup_enum_attack(pool: &mut Scheduler, args: Enum, config: &Arc<Config>) -> Result<usize> {
    let users = utils::load_list(&args.users)
        .context("Failed to load users")?;
    tinfo!("[+]", "loaded {} users", users.len());
    let scripts = utils::load_scripts(args.scripts, &config)
        .context("Failed to load scripts")?;
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

fn run_oneshot(oneshot: Oneshot, config: Arc<Config>) -> Result<()> {
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

fn main() -> Result<()> {
    let args = Args::from_args();

    let env = env_logger::Env::default();
    let env = match args.verbose {
        0 => env.filter_or("RUST_LOG", "warn"),
        1 => env.filter_or("RUST_LOG", "info"),
        _ => env.filter_or("RUST_LOG", "debug"),
    };
    env_logger::init_from_env(env);

    warn!("badtouch has been renamed to authoscope, please use the new binary name instead");

    if atty::isnt(atty::Stream::Stdout) {
        colored::control::SHOULD_COLORIZE.set_override(false);
    }

    let config = Arc::new(Config::load()?);
    #[cfg(target_os="linux")]
    authoscope::ulimit::set_nofile(&config)
        .context("Failed to set RLIMIT_NOFILE")?;

    let mut pool = Scheduler::new(args.workers);
    let mut report = Report::open(args.output)?;

    let attempts = match args.subcommand {
        SubCommand::Dict(dict) => setup_dictionary_attack(&mut pool, dict, &config)?,
        SubCommand::Creds(creds) => setup_credential_confirmation(&mut pool, creds, &config)?,
        SubCommand::Enum(enumerate) => setup_enum_attack(&mut pool, enumerate, &config)?,
        SubCommand::Oneshot(oneshot) => return run_oneshot(oneshot, config),
        SubCommand::Fsck(fsck) => return fsck::run_fsck(&args::Fsck {
            paths: fsck.paths,
            quiet: fsck.quiet,
            require_colon: fsck.require_colon,
            silent: fsck.silent,
        }),
        SubCommand::Completions(completions) => return completions.gen(),
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
                                scheduler::Creds::Enum(_) => {
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
