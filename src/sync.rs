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
use std::{
    collections::VecDeque,
    convert::TryFrom,
    fs::{self, DirEntry, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

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

        let mut unseen_dirs = VecDeque::new();
        unseen_dirs.push_back(self.src_root.clone());

        while let Some(ref src_dir) = unseen_dirs.pop_front() {
            let dst_dir = self.render_path(src_dir)?;
            let mut index_entries = vec![];

            for entry in self.dir_contents(src_dir)? {
                let file_type = entry.file_type()?;

                if file_type.is_file() {
                    if !should_render_path(&entry.path()) {
                        index_entries.push(IndexEntry::new(&entry, None));
                        continue;
                    }

                    let src_file = entry.path();
                    let dst_file = self.render_path(&src_file)?.with_extension("html");
                    println!("syncing: {}", src_file.display());

                    let html = self.render_file(&src_file)?;
                    File::create(&dst_file).and_then(|mut fh| fh.write_all(html.as_bytes()))?;
                    index_entries.push(IndexEntry::new(&entry, Some(dst_file)));
                } else if file_type.is_dir() {
                    let src_dir = entry.path();
                    let dst_dir = self.render_path(&src_dir)?;

                    unseen_dirs.push_back(src_dir);

                    if !dst_dir.exists() {
                        fs::create_dir(&dst_dir)?;
                    }
                    index_entries.push(IndexEntry::new(&entry, Some(dst_dir)));
                }
            }

            index_entries.sort();
            let index = Index::new(self, &src_dir, &dst_dir, &index_entries);
            File::create(&index.path())
                .and_then(|mut fh| fh.write_all(index.to_string().as_bytes()))?;
        }
        Ok(())
    }

    fn render_path(&self, path: &Path) -> io::Result<PathBuf> {
        replace_path_prefix(&self.src_root, &self.dst_root, path)
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

    fn dir_contents<'a>(
        &'a self,
        path: &'a Path,
    ) -> io::Result<impl Iterator<Item = DirEntry> + 'a> {
        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cannot find contents of non-directory path {}",
                    path.display()
                ),
            ));
        }
        Ok(fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(move |entry| !self.ignore_set.is_match(entry.path())))
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

fn should_render_path(path: &Path) -> bool {
    path.extension().unwrap_or_default() == "md"
}

fn replace_path_prefix(prefix: &Path, subst: &Path, path: &Path) -> io::Result<PathBuf> {
    path.strip_prefix(prefix)
        .map(|remaining| subst.join(remaining))
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cannot strip prefix {} of path {}",
                    prefix.display(),
                    path.display()
                ),
            )
        })
}
