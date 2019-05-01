use crate::{
    config::Config,
    render::{
        code::SyntaxHighlighter, index::Index, mathjax::MathjaxPolicy, stylesheet::Stylesheet,
        RenderOpts, ToHtml,
    },
};
use globset::GlobSet;
use object::{FileObject, Object};
use std::{
    convert::TryFrom,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use tree::DirTree;

pub mod object;

mod tree;

pub struct SyncOpts {
    /// Root source directory containing notes to be synced.
    pub src_root: PathBuf,
    /// Root destination directory which nodes from `src` will be synced to.
    pub dst_root: PathBuf,
    pub ignore: GlobSet,
    pub mathjax_policy: MathjaxPolicy,
    pub stylesheet: Option<Stylesheet>,
    pub syntax_highlighter: SyntaxHighlighter,
}

impl SyncOpts {
    pub fn sync(&self) -> io::Result<()> {
        if !self.dst_root.exists() {
            fs::create_dir_all(&self.dst_root)?;
        }

        let tree = DirTree::with_root(self.src_root.clone(), &self.dst_root, &self.ignore)?;
        for dir in tree.walk() {
            if !dir.render_path.exists() {
                fs::create_dir(&dir.render_path)?;
            }

            for child in &dir.children {
                match child {
                    Object::File(file) => {
                        if let Some(ref render_path) = file.render_path {
                            self.render(file, render_path)?;
                        }
                    }
                    _ => {}
                }
            }

            let index = Index::new(self, &dir)?;
            File::create(index.render_path())
                .and_then(|mut fh| fh.write_all(index.to_html().as_bytes()))?;
        }
        Ok(())
    }

    fn render(&self, file: &FileObject, render_path: &Path) -> io::Result<()> {
        println!("syncing: {}", file.path.display());
        let opts = self.render_opts();
        let html = file.read_content().and_then(|md| opts.render(&md))?;
        File::create(render_path).and_then(|mut fh| fh.write_all(html.as_bytes()))
    }

    #[inline]
    fn render_opts(&self) -> RenderOpts {
        RenderOpts::new(
            &self.stylesheet,
            &self.syntax_highlighter,
            &self.mathjax_policy,
        )
    }
}

impl TryFrom<Config> for SyncOpts {
    type Error = io::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let stylesheet_path = config.render.stylesheet_path.as_ref();
        let stylesheet = stylesheet_path
            .map(|path| Stylesheet::new(path, config.render.should_inline_stylesheet))
            .transpose()?;

        let syntax_highlighter = SyntaxHighlighter::with_theme(&config.render.code_block_theme)?;

        Ok(Self {
            src_root: config.sync.notes_dir,
            dst_root: config.sync.render_dir,
            ignore: config.sync.ignore,
            mathjax_policy: config.render.mathjax_policy,
            stylesheet,
            syntax_highlighter,
        })
    }
}
