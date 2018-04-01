use std::fs::{self, File};
use std::sync::Arc;
use std::io::{self, BufReader};
use std::io::prelude::*;
use errors::{Result, ResultExt};

use ctx;


pub fn load_list(path: &str) -> Result<Vec<Arc<String>>> {
    let f = File::open(path)?;
    let file = BufReader::new(&f);
    let lines: io::Result<_> = file.lines()
            .map(|x| x.map(|x| Arc::new(x)))
            .collect();
    Ok(lines?)
}

pub fn load_creds(path: &str) -> Result<Vec<(Arc<String>, Arc<String>)>> {
    let creds = load_list(&path)
                    .chain_err(|| "failed to load creds")?
                    .into_iter()
                    .map(|x| {
                        if let Some(idx) = x.find(":") {
                            let (user, password) = x.split_at(idx);
                            Ok((Arc::new(user.to_owned()), Arc::new(password[1..].to_owned())))
                        } else {
                            Err(format!("invalid list format: {:?}", x).into())
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
    Ok(creds)
}

pub fn load_scripts(paths: Vec<String>) -> Result<Vec<Arc<ctx::Script>>> {
    let mut scripts = Vec::new();

    for path in paths {
        let meta = fs::metadata(&path)?;

        if meta.is_dir() {
            for path in fs::read_dir(path)? {
                let path = path?.path();
                let path = path.to_str().unwrap();
                let script = Arc::new(ctx::Script::load(path)?);
                scripts.push(script);
            }
        } else {
            let script = Arc::new(ctx::Script::load(&path)?);
            scripts.push(script);
        }
    }

    Ok(scripts)
}
