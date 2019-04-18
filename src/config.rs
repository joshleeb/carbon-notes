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
                toml::from_str(&buf)
                    .map(|config: Config| config.resolve_stylesheet_path(path))
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("failed to parse config: {}", e),
                        )
                    })
            })
    }

    /// Map `stylesheet_path` to be relative to `config_path` if it is not absolute.
    fn resolve_stylesheet_path(mut self, config_path: &Path) -> Self {
        if let Some(ref stylesheet_path) = self.render.stylesheet_path {
            if !stylesheet_path.is_absolute() {
                let config_dir = config_path.ancestors().nth(1).unwrap();
                self.render.stylesheet_path = Some(config_dir.join(stylesheet_path));
            }
        }
        self
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SyncConfig {}

#[derive(Debug, Deserialize)]
pub(crate) struct RenderConfig {
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
