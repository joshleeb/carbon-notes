use crate::{
    config::Config,
    render::{
        code::SyntaxHighlighter, index::Index, mathjax::MathjaxPolicy, stylesheet::Stylesheet,
        RenderOpts, ToHtml,
    },
};
use globset::GlobSet;
use hash::ItemHashes;
use item::ItemType;
use std::{
    convert::TryFrom,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use walk::DirWalk;

pub mod item;

mod hash;
mod walk;

pub struct SyncOpts {
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
    pub fn sync(&self) -> io::Result<()> {
        if !self.dst_root.exists() {
            fs::create_dir_all(&self.dst_root)?;
        }

        // TODO: SyncOpts::sync should have a previous hash map and a new hash map so we can manage
        // entries that are deleted in a nicer way (i.e: not ignoring them).
        let hash_file = self.dst_root.join(hash::FILE_NAME);
        let hash_content = File::open(&hash_file).and_then(|mut fh| {
            let mut buf = String::new();
            fh.read_to_string(&mut buf).map(|_| buf)
        });
        let mut hashes = match hash_content {
            Ok(buf) => ItemHashes::try_from(buf.as_ref())?,
            _ => ItemHashes::default(),
        };

        for (source, dir_items) in DirWalk::from(self) {
            let render_dir = render_path(&source, &self.src_root, &self.dst_root)?;
            let mut index = Index::new(self, &source, &render_dir);

            for item in dir_items {
                match item.ty {
                    ItemType::File => {
                        if !item.should_render() {
                            index.push(item, false);
                            continue;
                        }
                        // TODO: SyncOpts::sync don't clone item for index
                        index.push(item.clone(), true);

                        let mut markdown = String::new();
                        File::open(&item.source)
                            .and_then(|mut fh| fh.read_to_string(&mut markdown))?;
                        if hashes.check_file(&item.source, &markdown) {
                            continue;
                        }
                        hashes.insert_file(item.source.clone(), &markdown);
                        println!("syncing: {}", item.source.display());

                        let html = self.render_file(&markdown)?;
                        File::create(&item.render)
                            .and_then(|mut fh| fh.write_all(html.as_bytes()))?;
                    }
                    ItemType::Directory => {
                        if !item.render.exists() {
                            fs::create_dir(&item.render)?;
                        }
                        index.push(item, true);
                    }
                    ItemType::Symlink => {}
                }
            }

            index.sort();
            File::create(&index.path())
                .and_then(|mut fh| fh.write_all(index.to_html().as_bytes()))?;
        }

        File::create(&hash_file).and_then(|mut fh| fh.write_all(hashes.to_string().as_bytes()))
    }

    fn render_file(&self, markdown: &str) -> io::Result<String> {
        let render = RenderOpts::new(
            &self.stylesheet,
            &self.syntax_highlighter,
            &self.mathjax_policy,
        );
        render.render(markdown)
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
