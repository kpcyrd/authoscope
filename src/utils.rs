use errors::{Result, ResultExt};

use std::str;
use std::fs::{self, File};
use std::sync::Arc;
use std::io::{self, BufReader};
use std::io::prelude::*;
use config::Config;

use ctx;


pub fn load_list(path: &str) -> Result<Vec<Arc<String>>> {
    let f = File::open(path)?;
    let file = BufReader::new(&f);
    let lines: io::Result<_> = file.lines()
            .map(|x| x.map(Arc::new))
            .collect();
    Ok(lines?)
}

pub fn load_creds(path: &str) -> Result<Vec<Arc<Vec<u8>>>> {
    let f = File::open(path)?;
    let mut file = BufReader::new(&f);

    let mut creds = Vec::new();

    let mut buf = Vec::new();
    const DELIM: u8 = b'\n';

    while 0 < file.read_until(DELIM, &mut buf)? {
        if buf[buf.len() - 1] == DELIM {
            buf.pop();
        }

        // ensure line is valid utf8
        str::from_utf8(&buf)
            .chain_err(|| "failed to decode utf8")?;

        if buf.iter().any(|x| *x == b':') {
            creds.push(Arc::new(buf.clone()));
        } else {
            return Err(format!("invalid list format: {:?}", buf).into())
        }

        buf.clear();
    }

    Ok(creds)
}

pub fn load_scripts(paths: Vec<String>, config: &Arc<Config>) -> Result<Vec<Arc<ctx::Script>>> {
    let mut scripts = Vec::new();

    for path in paths {
        let meta = fs::metadata(&path)?;

        if meta.is_dir() {
            for path in fs::read_dir(path)? {
                let path = path?.path();
                let path = path.to_str().unwrap();
                let script = Arc::new(ctx::Script::load(path, config.clone())?);
                scripts.push(script);
            }
        } else {
            let script = Arc::new(ctx::Script::load(&path, config.clone())?);
            scripts.push(script);
        }
    }

    Ok(scripts)
}
