use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

// TODO: config::Config should expand `~` and env variables in paths in config.toml
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub render: RenderConfig,
}

impl Config {
    pub(crate) fn read_or_write_default(path: &Path) -> io::Result<Self> {
        if path.exists() {
            return Config::from_toml(path);
        }

        let config = Config::default();
        let config_toml = config.to_toml()?;
        let path_dir = path.ancestors().nth(1).unwrap();

        fs::create_dir_all(&path_dir)?;
        File::create(&path)
            .and_then(|mut fh| fh.write_all(config_toml.as_bytes()))
            .map(|_| config)
    }

    fn from_toml(path: &Path) -> io::Result<Self> {
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

    fn to_toml(&self) -> io::Result<String> {
        toml::to_string_pretty(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse config: {}", e),
            )
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SyncConfig {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct RenderConfig {
    #[serde(rename = "stylesheet")]
    pub stylesheet_path: Option<PathBuf>,
    #[serde(rename = "inline_stylesheet")]
    pub should_inline_stylesheet: bool,
    pub code_block_theme: String,
    pub mathjax_policy: MathjaxPolicy,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            stylesheet_path: None,
            should_inline_stylesheet: false,
            code_block_theme: "base16-ocean.dark".into(),
            mathjax_policy: MathjaxPolicy::Always,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
