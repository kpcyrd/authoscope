extern crate hlua;
extern crate pbr;
extern crate docopt;
extern crate threadpool;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;

mod args;
mod ctx;
mod runtime;
mod tty;

use pbr::ProgressBar;
use error_chain::ChainedError;
use threadpool::ThreadPool;
use std::sync::mpsc;
use std::fs::{File};
use std::sync::Arc;
// use std::time::Duration;
use std::io;
use std::io::BufReader;
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
    paths.iter()
        .map(|path| {
            ctx::Script::load(path).map(|x| Arc::new(x))
        })
        .collect()
}

// this macro was vendored until https://github.com/a8m/pb/pull/62 is fixed
macro_rules! printfl {
   ($w:expr, $($tt:tt)*) => {{
        $w.write(&format!($($tt)*).as_bytes()).ok().expect("write() fail");
        $w.flush().ok().expect("flush() fail");
    }}
}

// replace this with pb.writeln after https://github.com/a8m/pb/pull/62
fn pb_writeln<W: Write>(pb: &mut ProgressBar<W>, s: &str) {
    let width = match tty::terminal_size() {
        Some((width, _)) => width.0 as usize,
        None => 80,
    };

    let mut out = format!("{}", s);
    if s.len() < width {
        out += &" ".repeat(width - s.len());
    }
    printfl!(io::stderr(), "\r{}\n", out);
    pb.tick();
}

fn run() -> Result<()> {
    let args = args::parse();

    let users = load_list(&args.arg_users).chain_err(|| "failed to load users")?;
    println!("[+] loaded {} users", users.len());
    let passwords = load_list(&args.arg_passwords).chain_err(|| "failed to load passwords")?;
    println!("[+] loaded {} passwords", passwords.len());
    let scripts = load_scripts(args.arg_scripts).chain_err(|| "failed to load scripts")?;
    println!("[+] loaded {} scripts", scripts.len());

    let attempts = users.len() * passwords.len() * scripts.len();

    let n_workers = 128;
    let pool = ThreadPool::new(n_workers);

    let (tx, rx) = mpsc::channel();

    println!("[*] submitting {} jobs to threadpool with {} workers", attempts, n_workers);
    for user in &users {
        for password in &passwords {
            for script in &scripts {
                let user = user.clone();
                let password = password.clone();
                let script = script.clone();
                let tx = tx.clone();
                pool.execute(move || {
                    let result = script.run_once(&user, &password);
                    tx.send((script, user, password, result)).expect("failed to send result");
                });
            }
        }
    }

    let mut pb = ProgressBar::new(attempts as u64);
    // we can't set this yet because we call .tick() in pb_writeln
    // pb_writeln usually would call .draw to bypass this, but this function is private
    // blocked by https://github.com/a8m/pb/pull/62
    // pb.set_max_refresh_rate(Some(Duration::from_millis(250)));
    pb.format("[#> ]");
    pb.tick();

    for (script, user, password, result) in rx.iter().take(attempts) {
        match result {
            Ok(valid) if !valid => (),
            Ok(_) => {
                pb_writeln(&mut pb, &format!("[+] valid({}) => {:?}:{:?}", script.descr(), user, password));
            },
            Err(err) => {
                pb_writeln(&mut pb, &format!("[!] error({}): {:?}", script.descr(), err));
            }
        };
        pb.inc();
    }
    pb.finish();

    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        eprintln!("{}", e.display_chain());
        std::process::exit(1);
    }
}
