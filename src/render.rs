pub(crate) use mathjax::MathjaxPolicy;

use clap::ArgMatches;
use pulldown_cmark::{html, Parser};
use std::{io, path::PathBuf};

mod mathjax;

#[derive(Debug)]
pub(crate) struct RenderOptions {
    stylesheet_path: Option<PathBuf>,
    should_inline_style: bool,
    mathjax_policy: MathjaxPolicy,
}

impl From<&ArgMatches<'static>> for RenderOptions {
    fn from(matches: &ArgMatches<'static>) -> Self {
        let stylesheet_path = matches.value_of("stylesheet").map(PathBuf::from);
        let mathjax_policy = matches.value_of("mathjax-policy").unwrap_or("auto");

        Self {
            stylesheet_path,
            should_inline_style: matches.is_present("inline-style"),
            mathjax_policy: mathjax_policy.parse().unwrap(),
        }
    }
}

/// Renders Markdown to HTML.
pub(crate) fn render(_opts: &RenderOptions, content: &str) -> io::Result<String> {
    // TODO: creating pulldown_cmark::parser in render::render
    //  - create parser with options
    //  - can create parser with callback for handling broken links
    let md_parser = Parser::new(content);

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, md_parser);
    Ok(html_buf)
}
