#![warn(unused_extern_crates)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate hlua_badtouch as hlua;
extern crate pbr;
extern crate threadpool;
extern crate colored;
extern crate time;
extern crate atty;
extern crate rand;
extern crate getch;
extern crate serde_json;
extern crate hyper;
extern crate kuchiki;
extern crate toml;
extern crate nix;
extern crate libc;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate structopt;

extern crate md5;
extern crate sha1;
extern crate sha2;
extern crate sha3;
extern crate digest;
extern crate hmac;
extern crate base64;

#[cfg(not(windows))]
extern crate termios;

extern crate reqwest;
extern crate mysql;
extern crate ldap3;

pub mod args;
pub mod config;
pub mod ctx;
pub mod fsck;
pub mod html;
pub mod http;
pub mod json;
pub mod keyboard;
pub mod pb;
pub mod runtime;
pub mod scheduler;
pub mod structs;
pub mod ulimit;
pub mod utils;


pub mod errors {
    use std;
    use hlua;
    use serde_json;
    use reqwest;
    use hyper;
    use base64;
    use toml;
    use nix;

    error_chain! {
        foreign_links {
            Io(std::io::Error);
            Lua(hlua::LuaError);
            Json(serde_json::Error);
            Reqwest(reqwest::Error);
            Hyper(hyper::error::Error);
            Utf8(std::str::Utf8Error);
            BufWrite(std::io::IntoInnerError<std::io::BufWriter<std::io::Stdout>>);
            Base64Decode(base64::DecodeError);
            Toml(toml::de::Error);
            Nix(nix::Error);
        }
    }
}
