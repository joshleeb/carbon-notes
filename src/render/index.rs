use crate::{
    render::{mathjax::MathjaxPolicy, template::Template, ToHtml},
    sync::{
        object::{DirObject, Object},
        SyncOpts,
    },
};
use maud::{html, Markup, Render};
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    io,
    path::{Path, PathBuf},
};

const INDEX_FILE_NAME: &str = "index.html";

pub struct Index<'a> {
    opts: &'a SyncOpts,
    dir: &'a DirObject,
}

impl<'a> Index<'a> {
    pub fn new(opts: &'a SyncOpts, dir: &'a DirObject) -> io::Result<Self> {
        Ok(Self { opts, dir })
    }

    pub fn render_path(&self) -> PathBuf {
        self.dir.render_path.join(INDEX_FILE_NAME)
    }

    fn title(&self) -> String {
        self.header()
    }

    fn header(&self) -> String {
        let relative = self.dir.path.strip_prefix(&self.opts.src_root).unwrap();
        format!("Index: /{}", relative.display())
    }
}

impl<'a> Render for Index<'a> {
    fn render(&self) -> Markup {
        let entries = self.dir.children.iter().map(IndexEntry::from);
        html! {
            h1 { (self.header()) }
            ul {
                @for entry in entries {
                    (entry)
                }
            }
        }
    }
}

impl<'a> ToHtml for Index<'a> {
    fn to_html(&self) -> String {
        Template {
            content: self.render(),
            title: &Some(self.title()),
            stylesheet: &self.opts.stylesheet,
            mathjax_policy: &MathjaxPolicy::Never,
        }
        .to_html()
    }
}

#[derive(Debug, Eq)]
pub struct IndexEntry<'a> {
    path: &'a Path,
    render_path: Option<PathBuf>,
}

impl<'a> IndexEntry<'a> {
    fn path(&'a self) -> &'a Path {
        if let Some(path) = &self.render_path {
            return path;
        }
        self.path
    }
}

impl<'a> From<&'a Object> for IndexEntry<'a> {
    // TODO: IndexEntry::from shouldn't need to clone fild.render_path.
    fn from(object: &'a Object) -> Self {
        let path = object.path();
        let render_path = match object {
            Object::Dir(dir) => Some(dir.render_path.join(INDEX_FILE_NAME)),
            Object::SourceFile(file) => Some(file.render_path.clone()),
            _ => None,
        };
        Self { path, render_path }
    }
}

impl<'a> Render for IndexEntry<'a> {
    fn render(&self) -> Markup {
        html! {
            li {
                a href=(self.path().display()) {
                    (self.path.display())
                }
            }
        }
    }
}

impl<'a> Ord for IndexEntry<'a> {
    fn cmp(&self, other: &IndexEntry) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl<'a> PartialOrd for IndexEntry<'a> {
    fn partial_cmp(&self, other: &IndexEntry) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for IndexEntry<'a> {
    fn eq(&self, other: &IndexEntry) -> bool {
        self.path == other.path && self.render_path == other.render_path
    }
}
