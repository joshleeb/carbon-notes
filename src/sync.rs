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
    source_dir: PathBuf,
    render_dir: PathBuf,
    ignore_set: GlobSet,
    mathjax_policy: MathjaxPolicy,
    stylesheet: Option<Stylesheet>,
    syntax_highlighter: SyntaxHighlighter,
}

impl SyncOpts {
    pub(crate) fn sync(&self) -> io::Result<()> {
        if !self.render_dir.exists() {
            fs::create_dir_all(&self.render_dir)?;
        }

        let mut unseen_dirs = VecDeque::new();
        unseen_dirs.push_back(self.source_dir.clone());

        while let Some(ref focus_dir) = unseen_dirs.pop_front() {
            // TODO: sync::sync check if index_entries with_capacity is better for performance
            //  - with size_hint from self.dir_contents
            let mut index_entries = vec![];
            let render_dir = self.render_dir_path(&focus_dir)?;

            for entry in self.dir_contents(focus_dir)? {
                let ft = entry.file_type()?;
                if ft.is_file() {
                    if !SyncOpts::should_render(&entry.path()) {
                        index_entries.push(IndexEntry::new(&entry, None));
                        continue;
                    }

                    let render_path = self.render_dir_path(&entry.path())?.with_extension("html");
                    let html = self.render_file(&entry.path())?;
                    File::create(&render_path).and_then(|mut fh| fh.write_all(html.as_bytes()))?;
                    index_entries.push(IndexEntry::new(&entry, Some(render_path)));
                    println!("{}", entry.path().display());
                } else if ft.is_dir() {
                    unseen_dirs.push_back(entry.path());

                    let render_path = self.render_dir_path(&entry.path())?;
                    if !render_path.exists() {
                        fs::create_dir(&render_path)?;
                    }
                    index_entries.push(IndexEntry::new(&entry, Some(render_path)));
                }
            }

            index_entries.sort();
            let index = Index::new(&render_dir, &index_entries, &self.stylesheet);
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

    fn render_dir_path(&self, path: &Path) -> io::Result<PathBuf> {
        path.strip_prefix(&self.source_dir)
            .map(|ext_path| self.render_dir.join(ext_path))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "cannot strip prefix {} of path {}",
                        self.source_dir.display(),
                        path.display()
                    ),
                )
            })
    }

    fn should_render(path: &Path) -> bool {
        path.extension().unwrap_or_default() == "md"
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
            source_dir: config.sync.source_dir,
            render_dir: config.sync.render_dir,
            ignore_set: config.sync.ignore,
            mathjax_policy: config.render.mathjax_policy,
            stylesheet,
            syntax_highlighter,
        })
    }
}
