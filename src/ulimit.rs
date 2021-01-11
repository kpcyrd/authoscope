use rlimit::{Resource, Rlim};
use crate::errors::*;
use crate::config::Config;


pub fn set_nofile(config: &Config) -> Result<()> {
    let (soft_limit, hard_limit) = rlimit::getrlimit(Resource::NOFILE)?;
    debug!("soft_limit={:?}, hard_limit={:?}", soft_limit, hard_limit);

    let new_hard_limit = if let Some(limit) = config.runtime.rlimit_nofile {
        Rlim::from_usize(limit)
    } else {
        hard_limit
    };
    info!("setting NOFILE limit to {:?}", new_hard_limit);
    rlimit::setrlimit(Resource::NOFILE, new_hard_limit, new_hard_limit)?;

    let (soft_limit, hard_limit) = rlimit::getrlimit(Resource::NOFILE)?;
    debug!("soft_limit={:?}, hard_limit={:?}", soft_limit, hard_limit);

    Ok(())
}
