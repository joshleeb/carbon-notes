use crate::sync::{
    incremental::DirStore,
    object::{DirObject, Object, SourceFileObject},
};
use globset::GlobSet;
use std::{
    collections::{hash_map::DefaultHasher, VecDeque},
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{self, Write},
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

        DirTree::compute_merkle_hash(&mut root_dir);
        Ok(Self { root: root_dir })
    }

    pub fn walk(&self) -> DirWalk {
        let mut unseen_dirs = VecDeque::new();
        unseen_dirs.push_back(&self.root);

        DirWalk { unseen_dirs }
    }

    pub fn persist_hashes(&mut self) -> io::Result<()> {
        for dir in self.walk() {
            let store = DirStore::from(dir.object);
            let json = store.to_json()?;
            if !dir.object.render_path.exists() {
                fs::create_dir(&dir.object.render_path)?;
            }
            File::create(store.path()).and_then(|mut fh| fh.write_all(json.as_bytes()))?;
        }
        Ok(())
    }

    /// Update hashes so that the hash of each directory is hashed with the hash of all it's child
    /// directories.
    fn compute_merkle_hash(root: &mut DirObject) {
        let mut hasher = DefaultHasher::new();
        root.contents_hash.hash(&mut hasher);

        for child in &mut root.children {
            match child {
                Object::Dir(ref mut child_dir) => {
                    DirTree::compute_merkle_hash(child_dir);
                    child_dir.merkle_hash.hash(&mut hasher);
                }
                Object::SourceFile(ref child_file) => child_file.contents_hash.hash(&mut hasher),
                _ => {}
            };
        }
        root.merkle_hash = hasher.finish();
    }
}

// TODO: tree::Dir should have a better name.
pub struct Dir<'a> {
    pub object: &'a DirObject,
    pub to_render: Vec<&'a SourceFileObject>,
    pub should_render_index: bool,
}

pub struct DirWalk<'a> {
    unseen_dirs: VecDeque<&'a DirObject>,
}

impl<'a> Iterator for DirWalk<'a> {
    type Item = Dir<'a>;

    // DirWalk::next should respect the `incremental` config value.
    fn next(&mut self) -> Option<Self::Item> {
        if self.unseen_dirs.is_empty() {
            return None;
        }
        let dir = self.unseen_dirs.pop_front().unwrap();
        let store = DirStore::read_for_dir(&dir.render_path);

        let mut to_render = vec![];
        for child in &dir.children {
            match child {
                Object::Dir(child_dir) => {
                    if !store.merkle_hash(dir.merkle_hash) {
                        self.unseen_dirs.push_back(child_dir)
                    }
                }
                Object::SourceFile(child_file) => {
                    if !store.source_file_content(&child_file.path, child_file.contents_hash) {
                        to_render.push(child_file)
                    }
                }
                _ => {}
            };
        }
        Some(Dir {
            object: dir,
            to_render,
            should_render_index: !store.dir_content(dir.contents_hash),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::object::{FileObject, LinkObject, SourceFileObject};

    #[test]
    fn compute_merkle_hash_dir() {
        let mut dir = DirObject::new(PathBuf::from("/a"), PathBuf::new());

        let mut hasher = DefaultHasher::new();
        dir.contents_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);

        assert_eq!(dir.merkle_hash, hasher.finish());
    }

    #[test]
    fn compute_merkle_hash_source_file() {
        let mut dir = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let mut file = SourceFileObject::new(PathBuf::from("/a/b.txt"), PathBuf::new());

        let file_contents_hash = 123321;
        file.contents_hash = file_contents_hash;

        dir.extend(vec![file.into()]);

        let mut hasher = DefaultHasher::new();
        dir.contents_hash.hash(&mut hasher);
        file_contents_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);
        assert_eq!(dir.merkle_hash, hasher.finish());
    }

    #[test]
    fn compute_merkle_hash_file() {
        let mut dir = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let file = FileObject::new(PathBuf::from("/a/b.txt"));

        dir.extend(vec![file.into()]);

        let mut hasher = DefaultHasher::new();
        dir.contents_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);
        assert_eq!(dir.merkle_hash, hasher.finish());
    }

    #[test]
    fn compute_merkle_hash_link() {
        let mut dir = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let link = LinkObject::new(PathBuf::from("/a/b.txt"));

        dir.extend(vec![link.into()]);

        let mut hasher = DefaultHasher::new();
        dir.contents_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);
        assert_eq!(dir.merkle_hash, hasher.finish());
    }

    #[test]
    fn compute_merkle_hash_empty_subdirs() {
        let mut root = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let subdir_b = DirObject::new(PathBuf::from("/a/b"), PathBuf::new());
        let subdir_c = DirObject::new(PathBuf::from("/a/c"), PathBuf::new());

        // Since the subdirs don't have any subdirs of themselves, the merkle hash will be equal to
        // the contents hash.
        let mut subdir_b_hasher = DefaultHasher::new();
        subdir_b.contents_hash.hash(&mut subdir_b_hasher);
        let mut subdir_c_hasher = DefaultHasher::new();
        subdir_c.contents_hash.hash(&mut subdir_c_hasher);

        root.extend(vec![subdir_b.into(), subdir_c.into()]);

        let mut root_hasher = DefaultHasher::new();
        root.contents_hash.hash(&mut root_hasher);
        subdir_b_hasher.finish().hash(&mut root_hasher);
        subdir_c_hasher.finish().hash(&mut root_hasher);

        DirTree::compute_merkle_hash(&mut root);
        assert_eq!(root.merkle_hash, root_hasher.finish());
    }

    // TODO: DirObject::compute_merkle_hash_subdir_file_path_change should be better.
    #[test]
    fn compute_merkle_hash_subdir_file_path_change() {
        let mut root = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let mut subdir_b = DirObject::new(PathBuf::from("/a/b"), PathBuf::new());
        let subdir_b_file = FileObject::new(PathBuf::from("/a/b/file.txt"));
        let subdir_c = DirObject::new(PathBuf::from("/a/c"), PathBuf::new());

        subdir_b.extend(vec![subdir_b_file.into()]);
        root.extend(vec![subdir_b.clone().into(), subdir_c.clone().into()]);

        DirTree::compute_merkle_hash(&mut root);

        let mut new_root = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let mut new_subdir_b = DirObject::new(PathBuf::from("/a/b"), PathBuf::new());
        let new_subdir_b_file = FileObject::new(PathBuf::from("/a/b/file-new.txt"));
        let new_subdir_c = DirObject::new(PathBuf::from("/a/c"), PathBuf::new());

        new_subdir_b.extend(vec![new_subdir_b_file.into()]);
        new_root.extend(vec![
            new_subdir_b.clone().into(),
            new_subdir_c.clone().into(),
        ]);

        DirTree::compute_merkle_hash(&mut new_root);

        assert_ne!(root.merkle_hash, new_root.merkle_hash);
        assert_eq!(root.contents_hash, new_root.contents_hash);
        assert_ne!(
            root.children[0].merkle_hash().unwrap(),
            new_root.children[0].merkle_hash().unwrap()
        );
        assert_ne!(
            root.children[0].contents_hash().unwrap(),
            new_root.children[0].contents_hash().unwrap(),
        );
        assert_eq!(
            root.children[1].merkle_hash().unwrap(),
            new_root.children[1].merkle_hash().unwrap(),
        );
        assert_eq!(
            root.children[1].contents_hash().unwrap(),
            new_root.children[1].contents_hash().unwrap(),
        );
    }

    // TODO: DirObject::compute_merkle_hash_subdir_file_content_change should be better.
    #[test]
    fn compute_merkle_hash_subdir_file_content_change() {
        let mut root = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let mut subdir_b = DirObject::new(PathBuf::from("/a/b"), PathBuf::new());
        let mut subdir_b_file =
            SourceFileObject::new(PathBuf::from("/a/b/file.txt"), PathBuf::new());
        subdir_b_file.contents_hash = 123321;
        let subdir_c = DirObject::new(PathBuf::from("/a/c"), PathBuf::new());

        subdir_b.extend(vec![subdir_b_file.into()]);
        root.extend(vec![subdir_b.clone().into(), subdir_c.clone().into()]);

        DirTree::compute_merkle_hash(&mut root);

        let mut new_root = DirObject::new(PathBuf::from("/a"), PathBuf::new());
        let mut new_subdir_b = DirObject::new(PathBuf::from("/a/b"), PathBuf::new());
        let mut new_subdir_b_file =
            SourceFileObject::new(PathBuf::from("/a/b/file.txt"), PathBuf::new());
        new_subdir_b_file.contents_hash = 789987;
        let new_subdir_c = DirObject::new(PathBuf::from("/a/c"), PathBuf::new());

        new_subdir_b.extend(vec![new_subdir_b_file.into()]);
        new_root.extend(vec![
            new_subdir_b.clone().into(),
            new_subdir_c.clone().into(),
        ]);

        DirTree::compute_merkle_hash(&mut new_root);

        assert_ne!(root.merkle_hash, new_root.merkle_hash);
        assert_eq!(root.contents_hash, new_root.contents_hash);
        assert_ne!(
            root.children[0].merkle_hash().unwrap(),
            new_root.children[0].merkle_hash().unwrap()
        );
        assert_eq!(
            root.children[0].contents_hash().unwrap(),
            new_root.children[0].contents_hash().unwrap(),
        );
        assert_eq!(
            root.children[1].merkle_hash().unwrap(),
            new_root.children[1].merkle_hash().unwrap(),
        );
        assert_eq!(
            root.children[1].contents_hash().unwrap(),
            new_root.children[1].contents_hash().unwrap(),
        );
    }
}
