use crate::config::Config;
use std::io;

pub(crate) fn sync(_config: &Config) -> io::Result<()> {
    Ok(())
}
