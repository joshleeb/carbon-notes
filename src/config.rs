use std::path::PathBuf;

#[derive(Debug, Default)]
pub(crate) struct Config {
    pub sync: SyncConfig,
    pub render: RenderConfig,
}

#[derive(Debug)]
pub(crate) struct RenderConfig {
    pub stylesheet_path: Option<PathBuf>,
    pub should_inline_stylesheet: bool,
    pub code_block_theme: &'static str,
    pub mathjax_policy: MathjaxPolicy,
}

impl Default for RenderConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap();
        Self {
            stylesheet_path: Some(home_dir.join(".config/carbon/github.css")),
            should_inline_stylesheet: false,
            code_block_theme: "base16-ocean.dark",
            mathjax_policy: MathjaxPolicy::Always,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum MathjaxPolicy {
    Always,
    // Never,
}

// TODO: config::MathjaxPolicy implement the `Auto` policy
impl MathjaxPolicy {
    pub(crate) fn inclusion(&self) -> bool {
        *self == MathjaxPolicy::Always
    }
}

#[derive(Debug)]
pub(crate) struct SyncConfig {
    pub source_dir: PathBuf,
    pub render_dir: PathBuf,
}

impl Default for SyncConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap();
        Self {
            source_dir: home_dir.join("Dropbox/notes"),
            render_dir: home_dir.join("Dropbox/notes/_rendered"),
        }
    }
}
