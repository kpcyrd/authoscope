use crate::errors::*;
use std::io::stdout;
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

#[inline]
pub fn parse() -> Args {
    Args::from_args()
}
