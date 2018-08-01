use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short = "v", long = "verbose",
                raw(global = "true"), parse(from_occurrences),
                help="Verbose output")]
    pub verbose: u8,
    #[structopt(short = "n", long = "workers", default_value = "16",
                help="Concurrent workers")]
    pub workers: usize,
    #[structopt(short = "o", long = "output",
                help="Write results to file")]
    pub output: Option<String>,
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(author = "",
                name="dict",
                about="Dictionary attack")]
    Dict(Dict),
    #[structopt(author = "",
                name="creds",
                about="Credential confirmation attack")]
    Creds(Creds),
    #[structopt(author = "",
                name="enum",
                about="Enumerate users")]
    Enum(Enum),
    #[structopt(author = "",
                name="oneshot",
                about="Test a single username-password combination")]
    Oneshot(Oneshot),
    #[structopt(author = "",
                name="fsck",
                about="Verify and fix encoding of a list")]
    Fsck(Fsck),
}

#[derive(StructOpt, Debug)]
pub struct Dict {
    #[structopt(help="Username list path")]
    pub users: String,
    #[structopt(help="Password list path")]
    pub passwords: String,
    #[structopt(raw(required="true"),
                help="Scripts to run")]
    pub scripts: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Creds {
    #[structopt(help="Credential list path")]
    pub creds: String,
    #[structopt(raw(required="true"),
                help="Scripts to run")]
    pub scripts: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Enum {
    #[structopt(help="Username list path")]
    pub users: String,
    #[structopt(raw(required="true"),
                help="Scripts to run")]
    pub scripts: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Oneshot {
    #[structopt(help="Script to run")]
    pub script: String,
    #[structopt(help="Username to test")]
    pub user: String,
    #[structopt(help="Password to test")]
    pub password: Option<String>,
}

#[derive(StructOpt, Debug)]
pub struct Fsck {
    #[structopt(short = "q", long = "quiet",
                help="Do not show invalid lines")]
    pub quiet: bool,
    #[structopt(short = "s", long = "silent",
                help="Do not show valid lines")]
    pub silent: bool,
    #[structopt(short = "c", long = "colon",
                help="Require one colon per line")]
    pub require_colon: bool,
    #[structopt(help="Files to read")]
    pub paths: Vec<String>,
}

pub fn parse() -> Args {
    Args::from_args()
}
