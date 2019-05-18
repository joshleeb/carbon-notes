use serde::{Deserialize, Serialize};

/// Merkle hash of the directory.
///
/// This is computed with the merkle hashes of any subdirectories in the directory.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleHash(u64);

impl From<u64> for MerkleHash {
    fn from(hash: u64) -> Self {
        Self(hash)
    }
}

impl From<MerkleHash> for u64 {
    fn from(hash: MerkleHash) -> Self {
        hash.0
    }
}

/// Hash of the contents of the directory.
///
/// This is computed without going into any subdirectories, and is purely at a surface level
/// which usually means just the source path of child is used.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DirChildrenHash(u64);

impl From<u64> for DirChildrenHash {
    fn from(hash: u64) -> Self {
        Self(hash)
    }
}

impl From<DirChildrenHash> for u64 {
    fn from(hash: DirChildrenHash) -> Self {
        hash.0
    }
}

/// Hash of the source files in the directory.
///
/// This is the content hash of each source file in the directory.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceContentsHash(u64);

impl From<u64> for SourceContentsHash {
    fn from(hash: u64) -> Self {
        Self(hash)
    }
}

impl From<SourceContentsHash> for u64 {
    fn from(hash: SourceContentsHash) -> Self {
        hash.0
    }
}
