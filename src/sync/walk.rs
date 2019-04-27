use crate::sync::{
    item::{Item, ItemType},
    SyncOpts,
};
use globset::GlobSet;
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct DirWalk<'a> {
    source_root: &'a Path,
    render_root: &'a Path,
    ignore_set: Option<&'a GlobSet>,
    unseen_dirs: VecDeque<PathBuf>,
}

impl<'a> DirWalk<'a> {
    fn new(source_root: &'a Path, render_root: &'a Path) -> Self {
        DirWalk {
            source_root,
            render_root,
            ignore_set: None,
            unseen_dirs: VecDeque::new(),
        }
    }

    fn with_ignore_set(self, ignore_set: &'a GlobSet) -> Self {
        DirWalk {
            ignore_set: Some(ignore_set),
            ..self
        }
    }

    fn read_dir(&self, path: &Path) -> Vec<Item> {
        fs::read_dir(path)
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|ref entry| Item::new(entry, &self.source_root, &self.render_root).ok())
            .filter(|ref item| !self.should_ignore(item))
            .collect()
    }

    fn push_dir(&mut self, dir: PathBuf) {
        self.unseen_dirs.push_back(dir);
    }

    fn should_ignore(&self, item: &Item) -> bool {
        self.ignore_set
            .map(|ignore| ignore.is_match(&item.source))
            .unwrap_or(false)
    }
}

impl<'a> From<&'a SyncOpts> for DirWalk<'a> {
    fn from(opts: &'a SyncOpts) -> Self {
        let mut walker =
            Self::new(&opts.src_root, &opts.dst_root).with_ignore_set(&opts.ignore_set);
        walker.push_dir(opts.src_root.clone());
        walker
    }
}

impl<'a> Iterator for DirWalk<'a> {
    type Item = (PathBuf, Vec<Item>);

    fn next(&mut self) -> Option<Self::Item> {
        self.unseen_dirs.pop_front().map(|source| {
            let mut items = vec![];
            for item in self.read_dir(&source) {
                if item.ty == ItemType::Directory {
                    self.push_dir(item.source.clone())
                }
                items.push(item);
            }
            (source, items)
        })
    }
}
