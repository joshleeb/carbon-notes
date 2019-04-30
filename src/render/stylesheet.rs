use crate::render::ToHtml;
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
    pub(crate) fn new(path: &Path, should_inline: bool) -> io::Result<Self> {
        if !should_inline {
            return Ok(Stylesheet::Link(path.into()));
        }
        let buf = File::open(path).and_then(|mut fh| {
            let mut buf = String::new();
            fh.read_to_string(&mut buf).map(|_| buf)
        })?;
        Ok(Self::Inline(buf))
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

impl ToHtml for Stylesheet {
    fn to_html(&self) -> String {
        self.render().into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_inline_template() {
        let styles = "body { background: red; }";
        let stylesheet = Stylesheet::Inline(styles.to_string());
        assert_eq!(stylesheet.to_html(), format!("<style>{}</style>", styles))
    }

    #[test]
    fn render_link_template() {
        let path = PathBuf::from("/path/to/stylesheet.css");
        let stylesheet = Stylesheet::Link(path.clone());
        assert_eq!(
            stylesheet.to_html(),
            format!(
                "<link rel=\"stylesheet\" type=\"text/css\" href=\"{}\"></link>",
                path.display()
            )
        )
    }
}
