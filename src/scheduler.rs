use ctx::Script;
use threadpool::ThreadPool;
use errors::Result;
use std::sync::mpsc;
use std::sync::Arc;

#[derive(Debug)]
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

type Msg = (Attempt, Result<bool>);

pub struct Scheduler {
    pool: ThreadPool,
    tx: mpsc::Sender<Msg>,
    rx: mpsc::Receiver<Msg>,
    inflight: usize,
}

impl Scheduler {
    #[inline]
    pub fn new(workers: usize) -> Scheduler {
        let (tx, rx) = mpsc::channel();
        Scheduler {
            pool: ThreadPool::new(workers),
            tx,
            rx,
            inflight: 0,
        }
    }

    #[inline]
    pub fn max_count(&self) -> usize {
        self.pool.max_count()
    }

    #[inline]
    pub fn has_work(&self) -> bool {
        self.inflight > 0
    }

    #[inline]
    pub fn run(&mut self, attempt: Attempt) {
        let tx = self.tx.clone();
        self.inflight += 1;
        self.pool.execute(move || {
            attempt.run(tx);
        });
    }

    #[inline]
    pub fn recv(&mut self) -> (Attempt, Result<bool>) {
        self.inflight -= 1;
        self.rx.recv().unwrap()
    }
}
