extern crate badtouch;
extern crate env_logger;
extern crate humantime;

use std::env;
use std::time::Instant;

fn main() {
    env_logger::init();

    let path = env::args().skip(1).next().expect("missing argument");

    let start = Instant::now();

    let creds = badtouch::utils::load_creds(&path)
                                    .expect("failed to load creds");

    let elapsed = start.elapsed();
    let average = elapsed / creds.len() as u32;
    println!("loaded {} records in {}, on average {}",
            creds.len(),
            humantime::format_duration(elapsed),
            humantime::format_duration(average),
    );
}
