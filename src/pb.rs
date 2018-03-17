// this file contains a wrapper around pbr to work around three things:
//
// - there is no function to write above the progress bar
// - .draw() isn't exposed so we can't bypass the ratelimit in tick.
//   This means we can't reliably redraw the graph after we wrote above it.
//   We have to implement rate limiting in our wrapper to ensure we are
//   able to bypass it when needed.
// - using colored strings breaks pbr
//
// https://github.com/a8m/pb/pull/62

use pbr;
use std::fmt::Display;
use std::io::prelude::*;
use std::io::{self, Stdout};
use time::{self, SteadyTime, Duration};


macro_rules! printfl {
   ($w:expr, $($tt:tt)*) => {{
        $w.write(&format!($($tt)*).as_bytes()).ok().expect("write() fail");
        $w.flush().ok().expect("flush() fail");
    }}
}

pub struct ProgressBar {
    pb: pbr::ProgressBar<Stdout>,
    current: u64,
    last_refresh_time: SteadyTime,
    max_refresh_rate: Option<time::Duration>,
}

impl ProgressBar {
    #[inline]
    pub fn new(total: u64) -> ProgressBar {
        let mut pb = pbr::ProgressBar::new(total);
        pb.format("(=> )");

        let now = SteadyTime::now();
        let refresh_rate = Duration::milliseconds(250);

        ProgressBar {
            pb,
            current: 0,
            last_refresh_time: now - refresh_rate,
            max_refresh_rate: Some(refresh_rate),
        }
    }

    #[inline]
    pub fn draw(&mut self) {
        self.pb.tick()
    }

    #[inline]
    pub fn writeln<T: Display>(&mut self, s: T) {
        printfl!(io::stderr(), "\r\x1B[2K{}\n", s);
        self.draw()
    }

    #[inline]
    pub fn tick(&mut self) {
        let now = SteadyTime::now();
        if let Some(mrr) = self.max_refresh_rate {
            if now - self.last_refresh_time < mrr {
                return;
            }
        }

        self.draw();

        self.last_refresh_time = SteadyTime::now();
    }

    #[inline]
    pub fn inc(&mut self) {
        let now = SteadyTime::now();
        if let Some(mrr) = self.max_refresh_rate {
            if now - self.last_refresh_time < mrr {
                self.current += 1;
                return;
            }
        }

        self.pb.set(self.current);

        self.last_refresh_time = SteadyTime::now();
    }

    #[inline]
    pub fn finish_replace<T: Display>(&self, s: T) {
        print!("\r\x1B[2K{}", s);
    }
}
