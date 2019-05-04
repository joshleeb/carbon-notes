use crate::sync::object::{DirObject, Object};
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
    /// Merkle hash of the directory.
    ///
    /// This is computed with the merkle hashes of any subdirectories in the directory.
    pub merkle_hash: u64,
    /// Hash of the contents of the directory.
    ///
    /// This is computed without going into any subdirectories, and is purely at a surface level
    /// which usually means just the source path of child is used.
    pub contents_hash: u64,
    /// Hash of the source files in the directory.
    ///
    /// This is the content hash of each source file in the directory.
    pub source_hash: HashMap<PathBuf, u64>,

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
                source_hash.insert(file.path.clone(), file.contents_hash);
            }
        }

        Self {
            merkle_hash: dir.merkle_hash,
            contents_hash: dir.contents_hash,
            source_hash,
            path: dir.render_path.join(DIR_HASH_FILE_NAME),
        }
    }
}

// TODO: DirStore2 should have a better name
pub struct DirStore2(Option<DirStore>);

// TODO: DirStore2 functions should have better names
impl DirStore2 {
    pub fn merkle_hash(&self, merkle_hash: u64) -> bool {
        self.0
            .as_ref()
            .map(|store| store.merkle_hash == merkle_hash)
            .unwrap_or(false)
    }

    pub fn dir_content(&self, contents_hash: u64) -> bool {
        self.0
            .as_ref()
            .map(|store| store.contents_hash == contents_hash)
            .unwrap_or(false)
    }

    pub fn source_file_content(&self, path: &Path, contents_hash: u64) -> bool {
        self.0
            .as_ref()
            .and_then(|store| store.source_hash.get(path))
            .map(|hash| *hash == contents_hash)
            .unwrap_or(false)
    }
}

impl From<Option<DirStore>> for DirStore2 {
    fn from(store: Option<DirStore>) -> Self {
        Self(store)
    }
}
