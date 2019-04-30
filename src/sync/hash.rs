use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    convert::TryFrom,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
    str,
};

pub(super) const FILE_NAME: &str = ".carbon-hashes";

#[derive(Debug, Default)]
pub struct ItemHashes {
    /// Map of the render path of the item to the hash.
    items: HashMap<PathBuf, u64>,
}

impl ItemHashes {
    pub fn insert_file(&mut self, path: PathBuf, buf: &str) {
        self.items.insert(path, self.file_hash(buf));
    }

    pub fn check_file(&self, path: &Path, buf: &str) -> bool {
        self.items
            .get(path)
            .map(|prev_hash| *prev_hash == self.file_hash(buf))
            .unwrap_or(false)
    }

    fn file_hash(&self, buf: &str) -> u64 {
        let mut hasher = Self::hasher();
        buf.hash(&mut hasher);
        hasher.finish()
    }

    fn hasher() -> impl Hasher {
        DefaultHasher::new()
    }
}

impl TryFrom<&str> for ItemHashes {
    type Error = io::Error;

    fn try_from(buf: &str) -> Result<Self, Self::Error> {
        let mut items = Self::default();
        let lines = buf.split("\n").filter(|line| !line.is_empty()).map(|line| {
            let components: Vec<&str> = line.splitn(2, " ").collect();
            (
                PathBuf::from(components[1]),
                components[0].parse::<u64>().unwrap(),
            )
        });
        for (path, hash) in lines {
            items.items.insert(path, hash);
        }
        Ok(items)
    }
}

impl ToString for ItemHashes {
    fn to_string(&self) -> String {
        self.items
            .iter()
            .map(|(path, hash)| format!("{} {}", hash, path.display()))
            .fold(String::new(), |mut acc, line| {
                acc.push('\n');
                acc.push_str(&line);
                acc
            })
    }
}
