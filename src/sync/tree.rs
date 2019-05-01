use crate::sync::object::{DirObject, Object};
use globset::GlobSet;
use std::{
    collections::VecDeque,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct DirTree {
    pub root: DirObject,
}

impl DirTree {
    pub fn with_root(root: PathBuf, render_root: &Path, ignore: &GlobSet) -> io::Result<Self> {
        let mut root_dir = match Object::new(root.clone(), &root, &render_root)? {
            Object::Dir(dir) => Ok(dir),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cannot create directory tree from file at path {}",
                    other.path().display()
                ),
            )),
        }?;

        let mut unseen_dirs = VecDeque::new();
        unseen_dirs.push_back(&mut root_dir);

        while !unseen_dirs.is_empty() {
            let dir = unseen_dirs.pop_front().unwrap();
            dir.extend(dir_children(&dir.path, &root, &render_root, ignore));

            for child in &mut dir.children {
                if let Object::Dir(child_dir) = child {
                    unseen_dirs.push_back(child_dir);
                }
            }
        }
        Ok(Self { root: root_dir })
    }

    pub fn walk(&self) -> DirWalk {
        let mut unseen_dirs = VecDeque::new();
        unseen_dirs.push_back(&self.root);

        DirWalk { unseen_dirs }
    }
}

pub struct DirWalk<'a> {
    unseen_dirs: VecDeque<&'a DirObject>,
}

impl<'a> Iterator for DirWalk<'a> {
    type Item = &'a DirObject;

    fn next(&mut self) -> Option<Self::Item> {
        self.unseen_dirs.pop_front().map(|dir| {
            for child in &dir.children {
                if let Object::Dir(child_dir) = child {
                    self.unseen_dirs.push_back(child_dir);
                }
            }
            dir
        })
    }
}

fn dir_children(
    path: &Path,
    source_root: &Path,
    render_root: &Path,
    ignore: &GlobSet,
) -> Vec<Object> {
    fs::read_dir(path)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|ref entry_path| !ignore.is_match(entry_path))
        .filter_map(|entry_path| Object::new(entry_path, source_root, render_root).ok())
        .collect()
}
