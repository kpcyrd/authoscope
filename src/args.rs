use structopt::StructOpt;
use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(author = "",
            raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct Args {
    #[structopt(help="Username list path")]
    pub users: String,
    #[structopt(help="Password list path")]
    pub passwords: String,
    #[structopt(help="Scripts to run")]
    pub scripts: Vec<String>,
    #[structopt(short = "n", long = "workers", default_value = "16",
                help="Concurrent workers")]
    pub workers: usize,
}

pub fn parse() -> Args {
    Args::from_args()
}
