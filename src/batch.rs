use errors::{Result};
use args;
use ctx::Script;
use config::Config;
use scheduler::{Attempt, Creds};

use std::sync::Arc;
use std::io::{self, BufReader, BufRead};
use serde_json;


#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    attempts: Vec<JsonAttempt>,
}

impl Job {
    pub fn run(self) -> JobResult {
        let mut results = JobResult::default();

        for attempt in self.attempts {
            // TODO: ensure debug output?
            match attempt.run() {
                Ok(success) => results.done.push((attempt.into(), success)),
                Err(err) => results.failed.push((attempt.into(), err.to_string())),
            }
        }

        results
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JobResult {
    done: Vec<(JsonAttempt, bool)>,
    failed: Vec<(JsonAttempt, String)>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JsonAttempt {
    pub user: String,
    pub password: String,
    pub script: String,
    pub ttl: u8,
}

impl JsonAttempt {
    #[inline]
    pub fn to_attempt(&self) -> Result<Attempt> {
        // TODO: this runs with default config
        // TODO: global user agent is discarded
        // TODO: keep in mind that the Config may contain sensitive data in the future
        let script = Script::load_from(self.script.as_bytes(),
                        Arc::new(Config::default()))?;

        // TODO: if possible, avoid clone
        Ok(Attempt {
            creds: Creds::new(self.user.clone(), self.password.clone()),
            script: Arc::new(script),
            ttl: self.ttl,
        })
    }

    #[inline]
    pub fn run(&self) -> Result<bool> {
        let attempt = self.to_attempt()?;
        attempt.run()
    }
}

impl From<Attempt> for JsonAttempt {
    fn from(attempt: Attempt) -> JsonAttempt {
        JsonAttempt {
            user: attempt.user().to_owned(),
            password: attempt.password().to_owned(),
            script: attempt.script.code().to_owned(),
            ttl: attempt.ttl,
        }
    }
}


pub fn run_batch(_args: args::Batch) -> Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    for line in reader.lines() {
        let line = line?;

        let job: Job = serde_json::from_str(&line)?;
        let results = job.run();

        let result = serde_json::to_string(&results)?;
        println!("{}", result);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_true() {
        let line = r#"
        {"attempts":[{"user":"foo","password":"bar","script":"descr = \"test\"\nfunction verify(user, password) return true end","ttl":15}]}
        "#;

        let job: Job = serde_json::from_str(line).unwrap();
        let result = job.run();

        assert_eq!(result, JobResult {
            done: vec![
                (JsonAttempt {
                    user: "foo".to_string(),
                    password: "bar".to_string(),
                    script: "descr = \"test\"\nfunction verify(user, password) return true end".to_string(),
                    ttl: 15
                }, true)
            ],
            failed: vec![],
        });
    }

    #[test]
    fn verify_false() {
        let line = r#"
        {"attempts":[{"user":"foo","password":"bar","script":"descr = \"test\"\nfunction verify(user, password) return false end","ttl":15}]}
        "#;

        let job: Job = serde_json::from_str(line).unwrap();
        let result = job.run();

        assert_eq!(result, JobResult {
            done: vec![
                (JsonAttempt {
                    user: "foo".to_string(),
                    password: "bar".to_string(),
                    script: "descr = \"test\"\nfunction verify(user, password) return false end".to_string(),
                    ttl: 15
                }, false)
            ],
            failed: vec![],
        });
    }

    #[test]
    fn verify_syntax_error() {
        let line = r#"
        {"attempts":[{"user":"foo","password":"bar","script":"descr = \"test\"\nfunction verify(user, passw","ttl":15}]}
        "#;

        let job: Job = serde_json::from_str(line).unwrap();
        let result = job.run();

        assert_eq!(result, JobResult {
            done: vec![],
            failed: vec![
                (JsonAttempt {
                    user: "foo".to_string(),
                    password: "bar".to_string(),
                    script: "descr = \"test\"\nfunction verify(user, passw".to_string(),
                    ttl: 15
                }, "Syntax error: [string \"chunk\"]:2: \')\' expected near <eof>".to_string())
            ],
        });
    }

    #[test]
    fn verify_runtime_error() {
        let line = r#"
        {"attempts":[{"user":"foo","password":"bar","script":"descr = \"test\"\nfunction verify(user, password) return \"an error\" end","ttl":15}]}
        "#;

        let job: Job = serde_json::from_str(line).unwrap();
        let result = job.run();

        assert_eq!(result, JobResult {
            done: vec![],
            failed: vec![
                (JsonAttempt {
                    user: "foo".to_string(),
                    password: "bar".to_string(),
                    script: "descr = \"test\"\nfunction verify(user, password) return \"an error\" end".to_string(),
                    ttl: 15
                }, "error: \"an error\"".to_string())
            ],
        });
    }
}
