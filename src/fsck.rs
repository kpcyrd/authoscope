use errors::Result;
use args::Fsck;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::str;


fn validate_file(path: &str) -> Result<()> {
    let f = File::open(path)?;
    let file = BufReader::new(&f);
    let mut out = BufWriter::new(io::stdout());

    let mut i = 1;
    for line in file.split(b'\n') {
        let line = line?;
        // TODO: filter empty lines(?)
        match str::from_utf8(&line) {
            Ok(line) => writeln!(&mut out, "{}", line)?,
            Err(_) => {
                eprintln!("Invalid(line {}): {:?} {:?}",
                    i,
                    String::from_utf8_lossy(&line),
                    line);
            },
        };
        i += 1;
    }

    // Close the BufWriter to flush it
    let _ = out.into_inner()?;

    Ok(())
}

pub fn run_fsck(args: Fsck) -> Result<()> {
    for path in args.paths {
        validate_file(&path)?;
    }
    Ok(())
}
