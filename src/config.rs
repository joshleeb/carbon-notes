use crate::render::mathjax::MathjaxPolicy;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Config {
    pub sync: SyncConfig,
    pub render: RenderConfig,
}

#[derive(Debug)]
pub struct RenderConfig {
    pub stylesheet_path: Option<PathBuf>,
    pub should_inline_stylesheet: bool,
    pub code_block_theme: String,
    pub mathjax_policy: MathjaxPolicy,
}

impl Default for RenderConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap();
        Self {
            stylesheet_path: Some(home_dir.join("code/carbon-notes/style/github.css")),
            should_inline_stylesheet: false,
            code_block_theme: String::from("base16-ocean.dark"),
            mathjax_policy: MathjaxPolicy::Always,
        }
    }
}

// TODO: config::SyncConfig should allow configuring a delete option which is to be implemented
//  - This would delete notes and dirs from the render dir that no longer exist in the source dir
#[derive(Debug)]
pub struct SyncConfig {
    pub notes_dir: PathBuf,
    pub render_dir: PathBuf,
    pub ignore: GlobSet,
    pub incremental: bool,
}

const GLOB_IGNORE: &[&str] = &[
    "*.tar.gz",
    ".directory",
    ".dropbox",
    ".dropbox.cache",
    ".git",
    ".mypy_cache",
    "_rendered",
    "target",
];

impl Default for SyncConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap();

        let mut ignore = GlobSetBuilder::new();
        for glob in GLOB_IGNORE
            .iter()
            .map(|s| Glob::new(&format!("**/{}", s)).unwrap())
        {
            ignore.add(glob);
        }

        Self {
            notes_dir: home_dir.join("Dropbox/store"),
            render_dir: home_dir.join("Documents/carbon/rendered"),
            ignore: ignore.build().unwrap(),
            incremental: true,
        }
    }
}
