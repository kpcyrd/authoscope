use errors::Result;
use args::Fsck;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::str;


fn validate_file(path: &str, args: &Fsck) -> Result<()> {
    let f = File::open(path)?;
    let file = BufReader::new(&f);
    let mut out = BufWriter::new(io::stdout());

    for (i, line) in file.split(b'\n').enumerate() {
        let line = line?;
        // TODO: filter empty lines(?)
        match str::from_utf8(&line) {
            Ok(line) => {
                if !args.require_colon || line.find(":").is_some() {
                    if !args.silent {
                        out.write(line.as_bytes())?;
                        out.write(b"\n")?;
                    }
                } else if !args.quiet {
                    eprintln!("Invalid(line {}): {:?}",
                        i,
                        line);
                }
            },
            Err(_) => {
                if !args.quiet {
                    eprintln!("Invalid(line {}): {:?} {:?}",
                        i,
                        String::from_utf8_lossy(&line),
                        line);
                }
            },
        };
    }

    // Close the BufWriter to flush it
    let _ = out.into_inner()?;

    Ok(())
}

pub fn run_fsck(args: Fsck) -> Result<()> {
    for path in &args.paths {
        validate_file(path, &args)?;
    }
    Ok(())
}
