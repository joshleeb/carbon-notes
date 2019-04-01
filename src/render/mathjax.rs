use std::{io, str::FromStr};

#[derive(Debug, PartialEq)]
pub(crate) enum MathjaxPolicy {
    Auto,
    Always,
    Never,
}

impl MathjaxPolicy {
    pub(crate) fn variants() -> &'static [&'static str] {
        &["auto", "always", "never"]
    }
}

impl FromStr for MathjaxPolicy {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(MathjaxPolicy::Auto),
            "always" => Ok(MathjaxPolicy::Always),
            "never" => Ok(MathjaxPolicy::Never),
            s => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown MathjaxPolicy {}", s),
            )),
        }
    }
}
