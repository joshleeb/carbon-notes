use serde::Deserialize;
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

// TODO: config::Config should allow for missing config `sync` and `render` and to use defaults
// instead
// TODO: config::Config should expand `~` and env variables in paths in config.toml
#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub sync: SyncConfig,
    pub render: RenderConfig,
}

impl Config {
    pub(crate) fn read_from(path: &Path) -> io::Result<Self> {
        let mut buf = String::new();
        File::open(path)
            .and_then(|mut fh| fh.read_to_string(&mut buf))
            .and_then(|_| {
                toml::from_str(&buf).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("failed to parse config: {}", e),
                    )
                })
            })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SyncConfig {}

#[derive(Debug, Deserialize)]
pub(crate) struct RenderConfig {
    // TODO: config::RenderConfig::stylesheet_path should be an absolute path that is relative to
    // config directory
    #[serde(rename = "stylesheet")]
    pub stylesheet_path: Option<PathBuf>,

    #[serde(rename = "inline_stylesheet")]
    #[serde(default = "default_inline_stylesheet")]
    pub should_inline_stylesheet: bool,

    #[serde(default = "default_code_block_theme")]
    pub code_block_theme: String,

    #[serde(default = "MathjaxPolicy::default")]
    pub mathjax_policy: MathjaxPolicy,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MathjaxPolicy {
    Always,
    Never,
}

// TODO: config::MathjaxPolicy implement the `Auto` policy
impl MathjaxPolicy {
    pub(crate) fn inclusion(&self) -> bool {
        *self == MathjaxPolicy::Always
    }
}

impl Default for MathjaxPolicy {
    fn default() -> Self {
        MathjaxPolicy::Always
    }
}

fn default_code_block_theme() -> String {
    "base16-ocean.dark".into()
}

fn default_inline_stylesheet() -> bool {
    true
}
