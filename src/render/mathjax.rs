use std::{io, str::FromStr};

// TODO: render::mathjax implement `Auto` policy
#[derive(Debug, PartialEq)]
pub(crate) enum MathjaxPolicy {
    Always,
    Never,
}

impl MathjaxPolicy {
    pub(crate) fn variants() -> &'static [&'static str] {
        &["always", "never"]
    }

    pub(crate) fn should_include(&self) -> bool {
        *self == MathjaxPolicy::Always
    }
}

impl FromStr for MathjaxPolicy {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always" => Ok(MathjaxPolicy::Always),
            "never" => Ok(MathjaxPolicy::Never),
            s => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown MathjaxPolicy {}", s),
            )),
        }
    }
}
