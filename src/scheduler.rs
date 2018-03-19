use ctx::Script;
use errors::Result;
use std::sync::mpsc;
use std::sync::Arc;

pub struct Attempt {
    pub user: Arc<String>,
    pub password: Arc<String>,
    pub script: Arc<Script>,
    pub ttl: u8,
}

impl Attempt {
    #[inline]
    pub fn new(user: &Arc<String>, password: &Arc<String>, script: &Arc<Script>) -> Attempt {
        Attempt {
            user: user.clone(),
            password: password.clone(),
            script: script.clone(),
            ttl: 5,
        }
    }

    #[inline]
    pub fn run(self, tx: mpsc::Sender<(Attempt, Result<bool>)>) {
        let result = self.script.run_once(&self.user, &self.password);
        tx.send((self, result)).expect("failed to send result");
    }
}
