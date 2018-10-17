#![warn(unused_extern_crates)]
extern crate hlua_badtouch as hlua;
extern crate pbr;
extern crate threadpool;
extern crate colored;
extern crate time;
extern crate atty;
extern crate rand;
extern crate getch;
extern crate serde_json;
extern crate kuchiki;
extern crate toml;
extern crate nix;
extern crate libc;
extern crate bufstream;
extern crate regex;
extern crate dirs;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate structopt;

extern crate md5;
extern crate sha1;
extern crate sha2;
extern crate sha3;
extern crate digest;
extern crate hmac;
extern crate base64;
extern crate bcrypt;

#[cfg(not(windows))]
extern crate termios;

extern crate reqwest;
extern crate mysql;
extern crate ldap3;
extern crate twox_hash;

pub mod args;
pub mod config;
pub mod ctx;
pub mod db;
pub mod fsck;
pub mod html;
pub mod http;
pub mod json;
pub mod keyboard;
pub mod pb;
pub mod runtime;
pub mod scheduler;
pub mod sockets;
pub mod structs;
pub mod ulimit;
pub mod utils;


pub mod errors {
    pub use failure::{Error, ResultExt};
    pub type Result<T> = ::std::result::Result<T, Error>;
}
