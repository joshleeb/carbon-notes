use crate::render::{mathjax::MathjaxPolicy, stylesheet::Stylesheet, template::Template};
use maud::{html, Markup, Render};
use std::{
    fs::{DirEntry, FileType},
    path::{Path, PathBuf},
};

const INDEX_FILE_NAME: &str = "index.html";

pub(crate) struct IndexEntry {
    source_path: PathBuf,
    render_path: Option<PathBuf>,
    file_type: FileType,
}

impl IndexEntry {
    pub(crate) fn new(entry: &DirEntry, render_path: Option<PathBuf>) -> Self {
        Self {
            source_path: entry.path(),
            file_type: entry.file_type().unwrap(),
            render_path,
        }
    }

    // TODO: IndexEntry::path rewrite to be easier to understand and without clones
    fn path(&self) -> PathBuf {
        self.render_path
            .clone()
            .map(|path| {
                if self.file_type.is_dir() {
                    path.join(INDEX_FILE_NAME)
                } else {
                    path
                }
            })
            .unwrap_or(self.source_path.clone())
    }
}

impl Render for IndexEntry {
    fn render(&self) -> Markup {
        html! {
            li {
                a href=(self.path().display()) {
                    (self.source_path.display())
                }
            }
        }
    }
}

pub(crate) struct Index<'a> {
    render_dir: &'a Path,
    entries: &'a Vec<IndexEntry>,
    stylesheet: &'a Option<Stylesheet>,
}

impl<'a> Index<'a> {
    pub(crate) fn new(
        render_dir: &'a Path,
        entries: &'a Vec<IndexEntry>,
        stylesheet: &'a Option<Stylesheet>,
    ) -> Self {
        Self {
            render_dir,
            entries,
            stylesheet,
        }
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.render_dir.join(INDEX_FILE_NAME)
    }

    // TODO: render::index::title implement.
    fn title(&self) -> String {
        self.render_dir.display().to_string()
    }
}

impl<'a> Render for Index<'a> {
    fn render(&self) -> Markup {
        html! {
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
            stylesheet: self.stylesheet,
            mathjax_policy: &MathjaxPolicy::Never,
        }
        .to_string()
    }
}
