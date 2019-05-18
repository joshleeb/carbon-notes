use crate::sync::{
    hash::MerkleHash,
    object::{DirObject, Object, SourceFileObject},
    store::{HashStore, HashStoreRw},
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
            let store = HashStore::from(dir.object);
            let json = store.to_json()?;
            if !dir.object.render_path.exists() {
                fs::create_dir(&dir.object.render_path)?;
            }
            File::create(HashStore::store_path(&dir.object.render_path))
                .and_then(|mut fh| fh.write_all(json.as_bytes()))?;
        }
        Ok(())
    }

    /// Update hashes so that the hash of each directory is hashed with the hash of all it's child
    /// directories.
    fn compute_merkle_hash(root: &mut DirObject) {
        let mut hasher = DefaultHasher::new();
        root.children_hash.hash(&mut hasher);

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
        root.merkle_hash = MerkleHash::from(hasher.finish());
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
        let store = HashStoreRw::read_dir(&dir.render_path);

        let mut to_render = vec![];
        for child in &dir.children {
            match child {
                Object::Dir(child_dir) => {
                    if !store.merkle_hash_eq(&dir.merkle_hash) {
                        self.unseen_dirs.push_back(child_dir)
                    }
                }
                Object::SourceFile(child_file) => {
                    if !store.source_hash_eq(&child_file.path, &child_file.contents_hash) {
                        to_render.push(child_file)
                    }
                }
                _ => {}
            };
        }
        Some(Dir {
            object: dir,
            to_render,
            should_render_index: !store.dir_hash_eq(&dir.children_hash),
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
    use crate::sync::object::{DirObject, FileObject, LinkObject, Object, SourceFileObject};

    fn obj2dir<'a>(object: &Object) -> &DirObject {
        if let Object::Dir(dir) = object {
            dir
        } else {
            panic!("expected DirObject, found {:?}", object);
        }
    }

    #[test]
    fn compute_merkle_hash_dir() {
        let mut dir = DirObject {
            path: "/a".into(),
            ..Default::default()
        };

        let mut hasher = DefaultHasher::new();
        dir.children_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);

        assert_eq!(dir.merkle_hash, MerkleHash::from(hasher.finish()));
    }

    #[test]
    fn compute_merkle_hash_source_file() {
        let mut dir = DirObject::from("/a");
        let file = SourceFileObject {
            path: "/a/b.txt".into(),
            contents_hash: 123321.into(),
            ..Default::default()
        };

        dir.extend(vec![file.clone().into()]);

        let mut hasher = DefaultHasher::new();
        dir.children_hash.hash(&mut hasher);
        file.contents_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);
        assert_eq!(dir.merkle_hash, MerkleHash::from(hasher.finish()));
    }

    #[test]
    fn compute_merkle_hash_file() {
        let mut dir = DirObject::from("/a");
        let file = FileObject::from("/a/b.txt");

        dir.extend(vec![file.into()]);

        let mut hasher = DefaultHasher::new();
        dir.children_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);
        assert_eq!(dir.merkle_hash, MerkleHash::from(hasher.finish()));
    }

    #[test]
    fn compute_merkle_hash_link() {
        let mut dir = DirObject::from("/a");
        let link = LinkObject::from("/a/b.txt");

        dir.extend(vec![link.into()]);

        let mut hasher = DefaultHasher::new();
        dir.children_hash.hash(&mut hasher);

        DirTree::compute_merkle_hash(&mut dir);
        assert_eq!(dir.merkle_hash, MerkleHash::from(hasher.finish()));
    }

    #[test]
    fn compute_merkle_hash_empty_subdirs() {
        let mut root = DirObject::from("/a");
        let subdir_b = DirObject::from("/a/b");
        let subdir_c = DirObject::from("/a/c");

        let mut subdir_b_hasher = DefaultHasher::new();
        subdir_b.children_hash.hash(&mut subdir_b_hasher);
        let mut subdir_c_hasher = DefaultHasher::new();
        subdir_c.children_hash.hash(&mut subdir_c_hasher);

        root.extend(vec![subdir_b.into(), subdir_c.into()]);

        let mut root_hasher = DefaultHasher::new();
        root.children_hash.hash(&mut root_hasher);
        subdir_b_hasher.finish().hash(&mut root_hasher);
        subdir_c_hasher.finish().hash(&mut root_hasher);

        DirTree::compute_merkle_hash(&mut root);
        assert_eq!(root.merkle_hash, MerkleHash::from(root_hasher.finish()));
        assert_ne!(u64::from(root.merkle_hash), u64::from(root.children_hash));
    }

    #[test]
    fn compute_merkle_hash_subdir_file_path_change() {
        let mut r1_root = DirObject::from("/a");
        let mut r1_subdir_b = DirObject::from("/a/b");
        let r1_subdir_b_file = FileObject::from("/a/b/file-round-1.txt");
        let r1_subdir_c = DirObject::from("/a/c");

        r1_subdir_b.extend(vec![r1_subdir_b_file.into()]);
        r1_root.extend(vec![r1_subdir_b.into(), r1_subdir_c.into()]);

        DirTree::compute_merkle_hash(&mut r1_root);

        let mut r2_root = DirObject::from("/a");
        let mut r2_subdir_b = DirObject::from("/a/b");
        let r2_subdir_b_file = FileObject::from("/a/b/file-round-2.txt");
        let r2_subdir_c = DirObject::from("/a/c");

        r2_subdir_b.extend(vec![r2_subdir_b_file.into()]);
        r2_root.extend(vec![r2_subdir_b.into(), r2_subdir_c.into()]);

        DirTree::compute_merkle_hash(&mut r2_root);

        assert_ne!(r1_root.merkle_hash, r2_root.merkle_hash);
        assert_eq!(r1_root.children_hash, r2_root.children_hash);
        assert_ne!(
            obj2dir(&r1_root.children[0]).merkle_hash,
            obj2dir(&r2_root.children[0]).merkle_hash
        );
        assert_ne!(
            obj2dir(&r1_root.children[0]).children_hash,
            obj2dir(&r2_root.children[0]).children_hash
        );
        assert_eq!(
            obj2dir(&r1_root.children[1]).merkle_hash,
            obj2dir(&r2_root.children[1]).merkle_hash
        );
        assert_eq!(
            obj2dir(&r1_root.children[1]).children_hash,
            obj2dir(&r2_root.children[1]).children_hash
        );
    }

    #[test]
    fn compute_merkle_hash_subdir_file_content_change() {
        let mut r1_root = DirObject::from("/a");
        let mut r1_subdir_b = DirObject::from("/a/b");
        let r1_subdir_b_file = SourceFileObject {
            path: "/a/b/file-round-1.txt".into(),
            contents_hash: 123321.into(),
            ..Default::default()
        };
        let r1_subdir_c = DirObject::from("/a/c");

        r1_subdir_b.extend(vec![r1_subdir_b_file.into()]);
        r1_root.extend(vec![r1_subdir_b.into(), r1_subdir_c.into()]);

        DirTree::compute_merkle_hash(&mut r1_root);

        let mut r2_root = DirObject::from("/a");
        let mut r2_subdir_b = DirObject::from("/a/b");
        let r2_subdir_b_file = SourceFileObject {
            path: "/a/b/file-round-1.txt".into(),
            contents_hash: 789987.into(),
            ..Default::default()
        };
        let r2_subdir_c = DirObject::from("/a/c");

        r2_subdir_b.extend(vec![r2_subdir_b_file.into()]);
        r2_root.extend(vec![r2_subdir_b.into(), r2_subdir_c.into()]);

        DirTree::compute_merkle_hash(&mut r2_root);

        assert_ne!(r1_root.merkle_hash, r2_root.merkle_hash);
        assert_eq!(r1_root.children_hash, r2_root.children_hash);
        assert_ne!(
            obj2dir(&r1_root.children[0]).merkle_hash,
            obj2dir(&r2_root.children[0]).merkle_hash
        );
        assert_eq!(
            obj2dir(&r1_root.children[0]).children_hash,
            obj2dir(&r2_root.children[0]).children_hash
        );
        assert_eq!(
            obj2dir(&r1_root.children[1]).merkle_hash,
            obj2dir(&r2_root.children[1]).merkle_hash
        );
        assert_eq!(
            obj2dir(&r1_root.children[1]).children_hash,
            obj2dir(&r2_root.children[1]).children_hash
        );
    }
}
