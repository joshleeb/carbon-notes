use crate::{
    render::{mathjax::MathjaxPolicy, template::Template, ToHtml},
    sync::{
        item::{Item, ItemType},
        SyncOpts,
    },
};
use maud::{html, Markup, Render};
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    path::{Path, PathBuf},
};

const INDEX_FILE_NAME: &str = "index.html";

pub(crate) struct Index<'a> {
    sync_opts: &'a SyncOpts,
    source: &'a Path,
    render: PathBuf,
    entries: Vec<IndexEntry>,
}

impl<'a> Index<'a> {
    pub(crate) fn new(sync_opts: &'a SyncOpts, source: &'a Path, render: &'a Path) -> Self {
        Self {
            sync_opts,
            source,
            render: render.join(INDEX_FILE_NAME),
            entries: vec![],
        }
    }

    pub(crate) fn push(&mut self, item: Item, is_rendered: bool) {
        self.entries.push(IndexEntry::new(item, is_rendered));
    }

    pub(crate) fn sort(&mut self) {
        self.entries.sort();
    }

    pub(crate) fn path(&self) -> &Path {
        self.render.as_ref()
    }

    fn title(&self) -> String {
        self.header()
    }

    fn header(&self) -> String {
        let relative = self.source.strip_prefix(&self.sync_opts.src_root).unwrap();
        format!("Index: /{}", relative.display())
    }
}

impl<'a> Render for Index<'a> {
    fn render(&self) -> Markup {
        html! {
            h1 { (self.header()) }
            ul {
                @for entry in &self.entries {
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
            stylesheet: &self.sync_opts.stylesheet,
            mathjax_policy: &MathjaxPolicy::Never,
        }
        .to_html()
    }
}

#[derive(Debug, Eq)]
pub(crate) struct IndexEntry {
    item: Item,
    is_rendered: bool,
}

impl IndexEntry {
    pub(crate) fn new(item: Item, is_rendered: bool) -> Self {
        Self { item, is_rendered }
    }

    // TODO: IndexEntry::path should not need to clone except for directory.
    fn path(&self) -> PathBuf {
        match (&self.item.ty, self.is_rendered) {
            (ItemType::File { .. }, true) => self.item.render.clone(),
            (ItemType::Directory, _) => self.item.render.join(INDEX_FILE_NAME),
            _ => self.item.source.clone(),
        }
    }
}

impl Render for IndexEntry {
    fn render(&self) -> Markup {
        html! {
            li {
                a href=(self.path().display()) {
                    (self.item.source.display())
                }
            }
        }
    }
}

impl Ord for IndexEntry {
    fn cmp(&self, other: &IndexEntry) -> Ordering {
        self.item.source.cmp(&other.item.source)
    }
}

impl PartialOrd for IndexEntry {
    fn partial_cmp(&self, other: &IndexEntry) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for IndexEntry {
    fn eq(&self, other: &IndexEntry) -> bool {
        self.item == other.item
    }
}
