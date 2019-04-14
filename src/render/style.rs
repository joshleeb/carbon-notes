use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

pub(crate) fn read_stylesheet(path: &Path) -> io::Result<String> {
    let mut buf = String::new();
    File::open(path)
        .and_then(|mut fh| fh.read_to_string(&mut buf))
        .map(|_| buf)
}
