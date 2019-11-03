// temporary solution until nix has ulimit support
// https://github.com/nix-rust/nix/pull/879
use std::mem;
use libc::{self, c_uint, rlimit, RLIM_INFINITY, rlim_t};
use crate::errors::Result;
use nix::errno::Errno;
use std::convert::TryInto;


#[derive(Debug)]
#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum Resource {
    RLIMIT_NOFILE = libc::RLIMIT_NOFILE,
}

pub fn getrlimit(resource: Resource) -> Result<(Option<rlim_t>, Option<rlim_t>)> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    let res = unsafe { libc::getrlimit((resource as c_uint).try_into().unwrap(), &mut rlim as *mut _) };
    let rlim = Errno::result(res).map(|_| {
        (if rlim.rlim_cur != RLIM_INFINITY { Some(rlim.rlim_cur) } else { None },
         if rlim.rlim_max != RLIM_INFINITY { Some(rlim.rlim_max) } else { None })
    })?;
    Ok(rlim)
}

pub fn setrlimit(resource: Resource, soft_limit: Option<rlim_t>, hard_limit: Option<rlim_t>) -> Result<()> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    rlim.rlim_cur = soft_limit.unwrap_or(RLIM_INFINITY);
    rlim.rlim_max = hard_limit.unwrap_or(RLIM_INFINITY);

    let res = unsafe { libc::setrlimit((resource as c_uint).try_into().unwrap(), &rlim as *const _) };
    Errno::result(res).map(drop)?;
    Ok(())
}
