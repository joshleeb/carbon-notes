use crate::sync::{
    hash::{DirChildrenHash, MerkleHash, SourceContentsHash},
    object::{DirObject, Object},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

const DIR_HASH_FILE_NAME: &str = ".carbon-hash-store.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct HashStore {
    pub merkle: MerkleHash,
    pub dir: DirChildrenHash,
    pub source: HashMap<PathBuf, SourceContentsHash>,
}

impl HashStore {
    pub fn to_json(&self) -> io::Result<String> {
        serde_json::to_string(&self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to serialize directory hash store: {}", e),
            )
        })
    }

    fn from_json(content: String) -> io::Result<Self> {
        serde_json::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to deserialize json into hash store: {}", e),
            )
        })
    }

    pub fn store_path(dir_path: &Path) -> PathBuf {
        dir_path.join(DIR_HASH_FILE_NAME)
    }
}

impl From<&DirObject> for HashStore {
    fn from(dir: &DirObject) -> Self {
        let mut source_hash = HashMap::new();
        for child in &dir.children {
            if let Object::SourceFile(file) = child {
                // DirStore::from shouldn't need to clone file.path.
                source_hash.insert(file.path.clone(), file.contents_hash.clone());
            }
        }

        Self {
            merkle: dir.merkle_hash.clone(),
            dir: dir.children_hash.clone(),
            source: source_hash,
        }
    }
}

pub struct HashStoreRw {
    store: Option<HashStore>,
}

impl HashStoreRw {
    pub fn read_dir(dir_path: &Path) -> Self {
        let path = HashStore::store_path(dir_path);
        File::open(&path)
            .and_then(|mut fh| {
                let mut content = String::new();
                fh.read_to_string(&mut content).map(|_| content)
            })
            .and_then(HashStore::from_json)
            .ok()
            .into()
    }

    pub fn merkle_hash_eq(&self, hash: &MerkleHash) -> bool {
        self.store
            .as_ref()
            .map(|store| store.merkle == *hash)
            .unwrap_or(false)
    }

    pub fn dir_hash_eq(&self, hash: &DirChildrenHash) -> bool {
        self.store
            .as_ref()
            .map(|store| store.dir == *hash)
            .unwrap_or(false)
    }

    pub fn source_hash_eq(&self, path: &Path, hash: &SourceContentsHash) -> bool {
        self.store
            .as_ref()
            .and_then(|store| store.source.get(path))
            .map(|source_hash| *source_hash == *hash)
            .unwrap_or(false)
    }
}

impl From<HashStore> for HashStoreRw {
    fn from(store: HashStore) -> Self {
        Some(store).into()
    }
}

impl From<Option<HashStore>> for HashStoreRw {
    fn from(store: Option<HashStore>) -> Self {
        Self { store }
    }
}
