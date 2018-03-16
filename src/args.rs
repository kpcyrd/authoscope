use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    pub users: String,
    pub passwords: String,
    pub scripts: Vec<String>,
}

pub fn parse() -> Args {
    Args::from_args()
}
