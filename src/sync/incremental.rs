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
pub struct DirStore {
    pub merkle: MerkleHash,
    pub dir: DirChildrenHash,
    pub source: HashMap<PathBuf, SourceContentsHash>,

    #[serde(skip)]
    path: PathBuf,
}

impl DirStore {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read_for_dir(path: &Path) -> DirStore2 {
        let store_path = path.join(DIR_HASH_FILE_NAME);
        File::open(&store_path)
            .and_then(|mut fh| {
                let mut content = String::new();
                fh.read_to_string(&mut content).map(|_| content)
            })
            .and_then(Self::from_json)
            .ok()
            .into()
    }

    pub fn to_json(&self) -> io::Result<String> {
        serde_json::to_string(&self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to serialize directory hash store: {}", e),
            )
        })
    }

    // TODO: DirStore::from_json shouldn't take String.
    fn from_json(content: String) -> io::Result<Self> {
        serde_json::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to deserialize json into hash store: {}", e),
            )
        })
    }
}

impl From<&DirObject> for DirStore {
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
            path: dir.render_path.join(DIR_HASH_FILE_NAME),
        }
    }
}

// TODO: DirStore2 should have a better name
pub struct DirStore2 {
    inner: Option<DirStore>,
}

impl DirStore2 {
    pub fn merkle_hash_eq(&self, hash: &MerkleHash) -> bool {
        self.inner
            .as_ref()
            .map(|store| store.merkle == *hash)
            .unwrap_or(false)
    }

    pub fn dir_hash_eq(&self, hash: &DirChildrenHash) -> bool {
        self.inner
            .as_ref()
            .map(|store| store.dir == *hash)
            .unwrap_or(false)
    }

    pub fn source_hash_eq(&self, path: &Path, hash: &SourceContentsHash) -> bool {
        self.inner
            .as_ref()
            .and_then(|store| store.source.get(path))
            .map(|source_hash| *source_hash == *hash)
            .unwrap_or(false)
    }
}

impl From<Option<DirStore>> for DirStore2 {
    fn from(inner: Option<DirStore>) -> Self {
        Self { inner }
    }
}
