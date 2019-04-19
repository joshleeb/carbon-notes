use maud::{html, Markup, PreEscaped, Render};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

pub(crate) enum Stylesheet {
    Inline(String),
    Link(PathBuf),
}

impl Stylesheet {
    pub(crate) fn from_config(path: &Path, should_inline: bool) -> io::Result<Self> {
        if !should_inline {
            return Ok(Stylesheet::Link(path.into()));
        }
        let mut buf = String::new();
        File::open(path).and_then(|mut fh| fh.read_to_string(&mut buf))?;
        Ok(Stylesheet::Inline(buf))
    }
}

impl Render for Stylesheet {
    fn render(&self) -> Markup {
        match self {
            Stylesheet::Inline(styles) => {
                html! { style { (PreEscaped(styles)) } }
            }
            Stylesheet::Link(path) => {
                let path_str = path.to_str().unwrap();
                html! { link rel="stylesheet" type="text/css" href=(path_str) {} }
            }
        }
    }
}
