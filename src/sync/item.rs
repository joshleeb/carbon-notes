use crate::sync;
use std::{
    convert::TryFrom,
    fs::DirEntry,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    pub source: PathBuf,
    pub render: PathBuf,
    pub ty: ItemType,
}

impl Item {
    pub(crate) fn new(
        entry: &DirEntry,
        source_root: &Path,
        render_root: &Path,
    ) -> io::Result<Self> {
        let source = entry.path();
        let ty = ItemType::try_from(entry)?;
        let render = sync::render_path(&source, source_root, render_root).map(|path| match ty {
            ItemType::File { .. } => path.with_extension("html"),
            _ => path,
        })?;

        Ok(Item {
            source,
            render,
            ty: ItemType::try_from(entry)?,
        })
    }

    pub(crate) fn should_render(&self) -> bool {
        match self.ty {
            ItemType::File { .. } => self.source.extension().unwrap_or_default() == "md",
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemType {
    File,
    Directory,
    Symlink,
}

impl TryFrom<&DirEntry> for ItemType {
    type Error = io::Error;

    fn try_from(entry: &DirEntry) -> Result<Self, Self::Error> {
        let ft = entry.file_type()?;
        if ft.is_file() {
            Ok(ItemType::File)
        } else if ft.is_dir() {
            Ok(ItemType::Directory)
        } else if ft.is_symlink() {
            Ok(ItemType::Symlink)
        } else {
            unreachable!()
        }
    }
}
