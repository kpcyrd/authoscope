#![warn(unused_extern_crates)]

use hlua_badtouch as hlua;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;


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
