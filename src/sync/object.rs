use crate::sync::hash::{DirChildrenHash, MerkleHash, SourceContentsHash};
use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Hash)]
pub enum Object {
    /// File that is renderable. For now this is only markdown files.
    SourceFile(SourceFileObject),
    /// File that is non-renderable.
    File(FileObject),
    Dir(DirObject),
    Symlink(LinkObject),
}

impl Object {
    pub fn new(path: PathBuf, source_root: &Path, render_root: &Path) -> io::Result<Self> {
        let ft = path.metadata()?.file_type();
        if ft.is_file() {
            if path.extension().unwrap_or_default() != "md" {
                return Ok(Self::File(FileObject::new(path)));
            }
            return SourceFileObject::with_source(path, source_root, render_root)
                .map(Self::SourceFile);
        }
        if ft.is_dir() {
            return Ok(Self::Dir(DirObject::with_source(
                path,
                source_root,
                render_root,
            )));
        }
        if ft.is_symlink() {
            return Ok(Self::Symlink(LinkObject::new(path)));
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("unknown filetype at path {}", path.display()),
        ))
    }

    pub fn path(&self) -> &Path {
        match self {
            Object::File(x) => x.path.as_ref(),
            Object::SourceFile(x) => x.path.as_ref(),
            Object::Dir(x) => x.path.as_ref(),
            Object::Symlink(x) => x.path.as_ref(),
        }
    }
}

impl From<SourceFileObject> for Object {
    fn from(file: SourceFileObject) -> Self {
        Self::SourceFile(file)
    }
}

impl From<FileObject> for Object {
    fn from(file: FileObject) -> Self {
        Self::File(file)
    }
}

impl From<DirObject> for Object {
    fn from(dir: DirObject) -> Self {
        Self::Dir(dir)
    }
}

impl From<LinkObject> for Object {
    fn from(link: LinkObject) -> Self {
        Self::Symlink(link)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SourceFileObject {
    pub path: PathBuf,
    pub render_path: PathBuf,
    pub contents_hash: SourceContentsHash,
}

impl SourceFileObject {
    // TODO: SourceFileObject::with_source should have a better name
    pub fn with_source(path: PathBuf, source_root: &Path, render_root: &Path) -> io::Result<Self> {
        let render_path = render_path(&path, source_root, render_root).with_extension("html");
        let contents_hash = SourceContentsHash::from(SourceFileObject::hash_contents(&path)?);

        Ok(SourceFileObject {
            path,
            render_path,
            contents_hash,
        })
    }

    pub fn read_content(&self) -> io::Result<String> {
        SourceFileObject::read_to_string(&self.path)
    }

    fn hash_contents(path: &Path) -> io::Result<u64> {
        let content = SourceFileObject::read_to_string(path)?;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn read_to_string(path: &Path) -> io::Result<String> {
        File::open(path).and_then(|mut fh| {
            let mut content = String::new();
            fh.read_to_string(&mut content).map(|_| content)
        })
    }
}

impl<T: Into<PathBuf>> From<T> for SourceFileObject {
    fn from(path: T) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}

impl Hash for SourceFileObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct FileObject {
    pub path: PathBuf,
}

impl FileObject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl<T: Into<PathBuf>> From<T> for FileObject {
    fn from(path: T) -> Self {
        Self { path: path.into() }
    }
}

impl Hash for FileObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, Default, Clone)]
pub struct DirObject {
    pub path: PathBuf,
    pub render_path: PathBuf,
    pub children: Vec<Object>,
    pub merkle_hash: MerkleHash,
    pub children_hash: DirChildrenHash,
}

impl DirObject {
    pub fn new(path: PathBuf, render_path: PathBuf) -> Self {
        Self {
            path,
            render_path,
            ..Default::default()
        }
    }

    // TODO: DirObject::with_source should have a better name
    pub fn with_source(path: PathBuf, source_root: &Path, render_root: &Path) -> Self {
        let render_path = render_path(&path, source_root, render_root);
        Self::new(path, render_path)
    }

    pub fn extend<I: IntoIterator<Item = Object>>(&mut self, children: I) {
        self.children.extend(children);

        let mut hasher = DefaultHasher::new();
        for child in &self.children {
            child.hash(&mut hasher);
        }
        self.children_hash = DirChildrenHash::from(hasher.finish());
    }
}

impl<T: Into<PathBuf>> From<T> for DirObject {
    fn from(path: T) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}

impl Hash for DirObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct LinkObject {
    pub path: PathBuf,
}

impl LinkObject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl<T: Into<PathBuf>> From<T> for LinkObject {
    fn from(path: T) -> Self {
        Self { path: path.into() }
    }
}

impl Hash for LinkObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

fn render_path(source: &Path, source_root: &Path, render_root: &Path) -> PathBuf {
    let path = source.strip_prefix(source_root).unwrap();
    render_root.join(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_render_path() {
        let source = PathBuf::from("/notes/projects/carbon.md");
        let source_root = PathBuf::from("/notes/");
        let render_root = PathBuf::from("/notes/_rendered/");
        assert_eq!(
            render_path(&source, &source_root, &render_root),
            PathBuf::from("/notes/_rendered/projects/carbon.md")
        );
    }

    #[test]
    fn file_render_path_adjacent() {
        let source = PathBuf::from("/notes/projects/carbon.md");
        let source_root = PathBuf::from("/notes/");
        let render_root = PathBuf::from("/docs/_rendered/");
        assert_eq!(
            render_path(&source, &source_root, &render_root),
            PathBuf::from("/docs/_rendered/projects/carbon.md")
        );
    }

    #[test]
    fn dir_render_path() {
        let source = PathBuf::from("/notes/projects/carbon");
        let source_root = PathBuf::from("/notes/");
        let render_root = PathBuf::from("/notes/_rendered/");
        assert_eq!(
            render_path(&source, &source_root, &render_root),
            PathBuf::from("/notes/_rendered/projects/carbon")
        );
    }

    #[test]
    fn dir_render_path_adjacent() {
        let source = PathBuf::from("/notes/projects/carbon");
        let source_root = PathBuf::from("/notes/");
        let render_root = PathBuf::from("/docs/_rendered/");
        assert_eq!(
            render_path(&source, &source_root, &render_root),
            PathBuf::from("/docs/_rendered/projects/carbon")
        );
    }

    mod dir_object {
        use super::*;

        #[test]
        fn extend_updates_contents_hash() {
            let mut dir = DirObject::new(PathBuf::new(), PathBuf::new());
            let children: Vec<Object> = vec![
                DirObject::from("/some/path").into(),
                SourceFileObject::from("/some/source-file-path").into(),
                FileObject::from("/some/file-path").into(),
                LinkObject::from("/some/link-path").into(),
            ];

            let mut hasher = DefaultHasher::new();
            for child in &children {
                child.hash(&mut hasher);
            }
            dir.extend(children);

            assert_eq!(dir.children.len(), 4);
            assert_eq!(dir.children_hash, hasher.finish().into());
        }
    }
}
