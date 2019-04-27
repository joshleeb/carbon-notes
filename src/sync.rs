use crate::{
    config::Config,
    render::{
        code::SyntaxHighlighter,
        index::{Index, IndexEntry},
        mathjax::MathjaxPolicy,
        stylesheet::Stylesheet,
        RenderOpts,
    },
};
use globset::GlobSet;
use item::ItemType;
use std::{
    convert::TryFrom,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use walk::DirWalk;

pub(crate) mod item;

mod walk;

pub(crate) struct SyncOpts {
    /// Root source directory containing notes to be synced.
    pub src_root: PathBuf,
    /// Root destination directory which nodes from `src` will be synced to.
    pub dst_root: PathBuf,
    pub ignore_set: GlobSet,
    pub mathjax_policy: MathjaxPolicy,
    pub stylesheet: Option<Stylesheet>,
    pub syntax_highlighter: SyntaxHighlighter,
}

impl SyncOpts {
    // TODO: SyncOpts::sync incremental syncing.
    pub(crate) fn sync(&self) -> io::Result<()> {
        if !self.dst_root.exists() {
            fs::create_dir_all(&self.dst_root)?;
        }

        for (source, dir_items) in DirWalk::from(self) {
            let render_dir = render_path(&source, &self.src_root, &self.dst_root)?;
            let mut index = Index::new(self, &source, &render_dir);

            for item in dir_items {
                match item.ty {
                    ItemType::File => {
                        if !should_render_path(&item.source) {
                            index.push(IndexEntry::new(item, false));
                            continue;
                        }
                        println!("syncing: {}", item.source.display());

                        let html = self.render_file(&item.source)?;
                        File::create(&item.render)
                            .and_then(|mut fh| fh.write_all(html.as_bytes()))?;
                        index.push(IndexEntry::new(item, true));
                    }
                    ItemType::Directory => {
                        if !item.render.exists() {
                            fs::create_dir(&item.render)?;
                        }
                        index.push(IndexEntry::new(item, true));
                    }
                    ItemType::Symlink => {}
                }
            }

            index.sort();
            File::create(&index.path())
                .and_then(|mut fh| fh.write_all(index.to_string().as_bytes()))?;
        }
        Ok(())
    }

    fn render_file(&self, path: &Path) -> io::Result<String> {
        let render = RenderOpts::new(
            &self.stylesheet,
            &self.syntax_highlighter,
            &self.mathjax_policy,
        );
        let mut markdown = String::new();
        File::open(path)
            .and_then(|mut fh| fh.read_to_string(&mut markdown))
            .and_then(|_| render.render(&markdown))
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
            ignore_set: config.sync.ignore,
            mathjax_policy: config.render.mathjax_policy,
            stylesheet,
            syntax_highlighter,
        })
    }
}

fn render_path(source_path: &Path, source_root: &Path, render_root: &Path) -> io::Result<PathBuf> {
    source_path
        .strip_prefix(source_root)
        .map(|path| render_root.join(path))
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cannot strip prefix {} of path {}",
                    source_root.display(),
                    source_path.display()
                ),
            )
        })
}

fn should_render_path(path: &Path) -> bool {
    path.extension().unwrap_or_default() == "md"
}
