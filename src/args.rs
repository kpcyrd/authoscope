use crate::errors::*;
use std::io::stdout;
use clap::{ArgAction, CommandFactory, Parser};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// Verbose output
    #[arg(short, long, global = true, action(ArgAction::Count))]
    pub verbose: u8,
    /// Concurrent workers
    #[arg(short = 'n', long, default_value = "16")]
    pub workers: usize,
    /// Write results to file
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// For each user try every password from a dictionary/wordlist
    Dict(Dict),
    /// Run a credential stuffing attack with a combolist
    Combo(Combo),
    /// For each user enumerate if an account exists with that name/email
    Enum(Enum),
    /// Run a script with a single username and password
    Run(Run),
    /// Verify a given input file is properly encoded and all entries have valid formatting
    Fsck(Fsck),
    Completions(Completions),
}

#[derive(Debug, Parser)]
pub struct Dict {
    /// Username list path
    pub users_path: PathBuf,
    /// Password list path
    pub passwords_path: PathBuf,
    /// Scripts to run
    #[arg(required=true)]
    pub scripts: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct Combo {
    /// Path to combolist
    pub path: PathBuf,
    /// Scripts to run
    #[arg(required=true)]
    pub scripts: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct Enum {
    /// Username list path
    pub users: String,
    /// Scripts to run
    #[arg(required=true)]
    pub scripts: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct Run {
    /// Script to run
    pub script: String,
    /// Username to test
    pub user: String,
    /// Password to test
    pub password: Option<String>,
    /// Set the exitcode to 2 if the credentials are invalid
    #[arg(short = 'x', long)]
    pub exitcode: bool,
}

#[derive(Debug, Parser)]
pub struct Fsck {
    /// Do not show invalid lines
    #[arg(short = 'q', long)]
    pub quiet: bool,
    /// Do not show valid lines
    #[arg(short = 's', long)]
    pub silent: bool,
    /// Require one colon per line
    #[arg(short = 'c', long = "colon")]
    pub require_colon: bool,
    /// Files to read
    pub paths: Vec<String>,
}

/// Generate shell completions
#[derive(Debug, Parser)]
pub struct Completions {
    pub shell: Shell,
}

pub fn gen_completions(args: &Completions) -> Result<()> {
    clap_complete::generate(args.shell, &mut Args::command(), "authoscope", &mut stdout());
    Ok(())
}
