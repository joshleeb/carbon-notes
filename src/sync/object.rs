use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Object {
    File(FileObject),
    Dir(DirObject),
    Symlink(LinkObject),
}

impl Object {
    pub fn new(path: PathBuf, source_root: &Path, render_root: &Path) -> io::Result<Self> {
        let ft = path.metadata()?.file_type();
        if ft.is_file() {
            let render_path = FileObject::render_path(&path, source_root, render_root);
            return Ok(Self::File(FileObject::new(path, render_path)));
        }
        if ft.is_dir() {
            let render_path = DirObject::render_path(&path, source_root, render_root);
            return Ok(Self::Dir(DirObject::empty(path, render_path)));
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
            Object::Dir(x) => x.path.as_ref(),
            Object::Symlink(x) => x.path.as_ref(),
        }
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

#[derive(Debug)]
pub struct FileObject {
    pub path: PathBuf,
    pub render_path: Option<PathBuf>,
}

impl FileObject {
    fn new(path: PathBuf, render_path: Option<PathBuf>) -> Self {
        Self { path, render_path }
    }

    pub fn read_content(&self) -> io::Result<String> {
        File::open(&self.path).and_then(|mut fh| {
            let mut content = String::new();
            fh.read_to_string(&mut content).map(|_| content)
        })
    }

    fn render_path(path: &Path, source_root: &Path, render_root: &Path) -> Option<PathBuf> {
        if path.extension().unwrap_or_default() != "md" {
            return None;
        }
        Some(render_path(path, source_root, render_root).with_extension("html"))
    }
}

#[derive(Debug)]
pub struct DirObject {
    pub path: PathBuf,
    pub render_path: PathBuf,
    pub children: Vec<Object>,
}

impl DirObject {
    pub fn empty(path: PathBuf, render_path: PathBuf) -> Self {
        Self {
            path,
            render_path,
            children: vec![],
        }
    }

    pub fn extend<I: IntoIterator<Item = Object>>(&mut self, children: I) {
        self.children.extend(children);
    }

    fn render_path(path: &Path, source_root: &Path, render_root: &Path) -> PathBuf {
        render_path(&path, source_root, render_root)
    }
}

#[derive(Debug)]
pub struct LinkObject {
    pub path: PathBuf,
}

impl LinkObject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
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
}
