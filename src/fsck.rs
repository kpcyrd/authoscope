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
    let mut file = BufReader::new(&f);
    let mut out = BufWriter::new(io::stdout());

    let mut i = 0;
    let mut buf = Vec::new();
    const DELIM: u8 = b'\n';

    while 0 < file.read_until(DELIM, &mut buf)? {
        /*
        not removing the \n so we don't have to append it later
        if buf[buf.len() - 1] == DELIM {
            buf.pop();
        }
        */
        // TODO: remove empty lines?

        match str::from_utf8(&buf) {
            Ok(line) => {
                if !args.require_colon || buf.iter().any(|x| *x == b':') {
                    if !args.silent {
                        out.write_all(line.as_bytes())?;
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
                        String::from_utf8_lossy(&buf),
                        buf);
                }
            },
        };

        buf.clear();
        i += 1;
    }

    // Close the BufWriter to flush it
    let _ = out.into_inner()?;

    Ok(())
}

pub fn run_fsck(args: &Fsck) -> Result<()> {
    for path in &args.paths {
        validate_file(path, &args)?;
    }
    Ok(())
}
