use crate::render::mathjax::MathjaxPolicy;
use globset::{Glob, GlobSet, GlobSetBuilder};
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
pub(crate) struct SyncConfig {
    pub source_dir: PathBuf,
    pub render_dir: PathBuf,
    pub ignore: GlobSet,
}

const GLOB_IGNORE: [&str; 4] = ["_rendered", ".directory", ".dropbox", ".dropbox.cache"];

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
            source_dir: home_dir.join("Dropbox/notes"),
            render_dir: home_dir.join("Dropbox/notes/_rendered"),
            ignore: ignore.build().unwrap(),
        }
    }
}
