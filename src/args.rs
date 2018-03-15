use docopt::Docopt;

const USAGE: &'static str = "
Foobar

Usage:
  badtouch <users> <passwords> <scripts> ...
";

#[derive(Debug, Deserialize)]
pub struct Args {
    pub arg_users: String,
    pub arg_passwords: String,
    pub arg_scripts: Vec<String>,
}

pub fn parse() -> Args {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    args
}
