use crate::{
    render::{mathjax::MathjaxPolicy, template::Template},
    sync::SyncOpts,
};
use maud::{html, Markup, Render};
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    fs::{DirEntry, FileType},
    path::{Path, PathBuf},
};

const INDEX_FILE_NAME: &str = "index.html";

#[derive(Eq)]
pub(crate) struct IndexEntry {
    src: PathBuf,
    dst: Option<PathBuf>,
    file_type: FileType,
}

impl IndexEntry {
    pub(crate) fn new(entry: &DirEntry, dst: Option<PathBuf>) -> Self {
        Self {
            src: entry.path(),
            file_type: entry.file_type().unwrap(),
            dst,
        }
    }

    // TODO: IndexEntry::path rewrite to be easier to understand and without clones
    fn path(&self) -> PathBuf {
        self.dst
            .clone()
            .map(|path| {
                if self.file_type.is_dir() {
                    path.join(INDEX_FILE_NAME)
                } else {
                    path
                }
            })
            .unwrap_or(self.src.clone())
    }
}

impl Render for IndexEntry {
    fn render(&self) -> Markup {
        html! {
            li {
                a href=(self.path().display()) {
                    (self.src.display())
                }
            }
        }
    }
}

impl Ord for IndexEntry {
    fn cmp(&self, other: &IndexEntry) -> Ordering {
        self.src.cmp(&other.src)
    }
}

impl PartialOrd for IndexEntry {
    fn partial_cmp(&self, other: &IndexEntry) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for IndexEntry {
    fn eq(&self, other: &IndexEntry) -> bool {
        self.src == other.src && self.file_type == other.file_type
    }
}

pub(crate) struct Index<'a> {
    sync_opts: &'a SyncOpts,
    src_dir: &'a Path,
    dst_dir: &'a Path,
    entries: &'a Vec<IndexEntry>,
}

impl<'a> Index<'a> {
    pub(crate) fn new(
        sync_opts: &'a SyncOpts,
        src_dir: &'a Path,
        dst_dir: &'a Path,
        entries: &'a Vec<IndexEntry>,
    ) -> Self {
        Self {
            sync_opts,
            src_dir,
            dst_dir,
            entries,
        }
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.dst_dir.join(INDEX_FILE_NAME)
    }

    fn title(&self) -> String {
        self.header()
    }

    // TODO: render::index::title implement.
    fn header(&self) -> String {
        let relative = self.src_dir.strip_prefix(&self.sync_opts.src_root).unwrap();
        format!("Index: /{}", relative.display())
    }
}

impl<'a> Render for Index<'a> {
    fn render(&self) -> Markup {
        html! {
            h1 { (self.header()) }
            ul {
                @for entry in self.entries {
                    (entry)
                }
            }
        }
    }
}

impl<'a> ToString for Index<'a> {
    fn to_string(&self) -> String {
        Template {
            content: self.render(),
            title: &Some(self.title()),
            stylesheet: &self.sync_opts.stylesheet,
            mathjax_policy: &MathjaxPolicy::Never,
        }
        .to_string()
    }
}
