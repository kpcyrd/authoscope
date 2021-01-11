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
use atty;
use colored::Colorize;
use std::fmt::Display;
use std::io::prelude::*;
use std::io::{self, Stdout};
use time::{self, Instant, Duration};


macro_rules! printfl {
   ($w:expr, $($tt:tt)*) => {{
        $w.write(&format!($($tt)*).as_bytes()).ok().expect("write() fail");
        $w.flush().ok().expect("flush() fail");
    }}
}

pub struct ProgressBar {
    pb: pbr::ProgressBar<Stdout>,
    current: u64,
    last_refresh_time: Instant,
    max_refresh_rate: Option<time::Duration>,
    atty: bool,
}

impl ProgressBar {
    #[inline]
    pub fn new(total: u64) -> ProgressBar {
        let mut pb = pbr::ProgressBar::new(total);
        pb.format("(=> )");

        let now = Instant::now();
        let refresh_rate = Duration::milliseconds(250);
        let atty = atty::is(atty::Stream::Stdout);

        ProgressBar {
            pb,
            current: 0,
            last_refresh_time: now - refresh_rate,
            max_refresh_rate: Some(refresh_rate),
            atty,
        }
    }

    #[inline]
    pub fn draw(&mut self) {
        if !self.atty {
            return;
        }

        self.pb.tick()
    }

    #[inline]
    pub fn print_help(&mut self) {
        self.writeln(format!("{} {}", "[+]".bold(),
            "[h] help, [p] pause, [r] resume, [+] increase threads, [-] decrease threads".dimmed()));
    }

    #[inline]
    pub fn writeln<T: Display>(&mut self, s: T) {
        printfl!(io::stderr(), "\r\x1B[2K{}\n", s);
        self.draw()
    }

    #[inline]
    pub fn tick(&mut self) {
        let now = Instant::now();
        if let Some(mrr) = self.max_refresh_rate {
            if now - self.last_refresh_time < mrr {
                return;
            }
        }

        self.draw();

        self.last_refresh_time = Instant::now();
    }

    #[inline]
    pub fn inc(&mut self) {
        if !self.atty {
            return;
        }

        let now = Instant::now();
        if let Some(mrr) = self.max_refresh_rate {
            if now - self.last_refresh_time < mrr {
                self.current += 1;
                return;
            }
        }

        self.pb.set(self.current);

        self.last_refresh_time = Instant::now();
    }

    #[inline]
    pub fn finish_replace<T: Display>(&self, s: T) {
        if self.atty {
            print!("\r\x1B[2K{}", s);
        } else {
            print!("{}", s);
        }
    }
}
