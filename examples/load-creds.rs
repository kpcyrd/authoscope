use std::env;
use std::time::Instant;
use authoscope::errors::*;

fn main() -> Result<()> {
    env_logger::init();

    let path = env::args().nth(1)
        .context("Missing argument")?;

    let start = Instant::now();

    let creds = authoscope::utils::load_combolist(path)
        .context("Failed to load creds")?;

    let elapsed = start.elapsed();
    let average = elapsed / creds.len() as u32;
    println!("loaded {} records in {}, on average {}",
            creds.len(),
            humantime::format_duration(elapsed),
            humantime::format_duration(average),
    );

    Ok(())
}
