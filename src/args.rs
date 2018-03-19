use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(short = "n", long = "workers", default_value = "16",
                help="Concurrent workers")]
    pub workers: usize,
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
}

#[derive(StructOpt, Debug)]
pub struct Dict {
    #[structopt(help="Username list path")]
    pub users: String,
    #[structopt(help="Password list path")]
    pub passwords: String,
    #[structopt(help="Scripts to run")]
    pub scripts: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Creds {
    #[structopt(help="Credential list path")]
    pub creds: String,
    #[structopt(help="Scripts to run")]
    pub scripts: Vec<String>,
}

pub fn parse() -> Args {
    Args::from_args()
}
