pub(crate) use mathjax::MathjaxPolicy;

use self::code::{CodeBlock, SyntaxHighlighter};
use clap::ArgMatches;
use pulldown_cmark::{html, Event, Parser, Tag};
use regex::Regex;
use std::{io, path::PathBuf};

mod code;
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

#[derive(Default)]
struct RenderState {
    /// ATX header level if a header is being processed.
    header: Option<i32>,
    /// Code block if a code block is being processed.
    code_block: Option<CodeBlock>,
}

/// Renders Markdown to HTML.
pub(crate) fn render(opts: &RenderOptions, content: &str) -> io::Result<String> {
    // TODO: creating pulldown_cmark::parser in render::render
    //  - create parser with options
    //  - can create parser with callback for handling broken links
    let md_parser = Parser::new(content);
    let syntax_highlighter = SyntaxHighlighter::default();

    let mut state = RenderState::default();
    let mut events = vec![];

    for event in md_parser {
        match event {
            Event::Start(Tag::Header(atx_level)) => {
                state.header = Some(atx_level);
            }
            Event::Start(Tag::CodeBlock(language)) => {
                state.code_block = Some(CodeBlock::new(language));
            }
            Event::Text(text) => {
                if let Some(atx_level) = state.header {
                    events.push(render_header_start(atx_level, &text));
                    state.header = None;
                }
                if let Some(ref mut code_block) = state.code_block {
                    code_block.push(&text);
                } else {
                    events.push(Event::Text(text));
                }
            }
            Event::End(Tag::CodeBlock(_)) => {
                if let Some(code_block) = state.code_block {
                    events.push(syntax_highlighter.render(&code_block));
                    state.code_block = None;
                }
            }
            ev => events.push(ev),
        }
    }

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, events.into_iter());
    Ok(html_buf)
}

/// Render a header start event or an HTML tag for the header with an ID.
fn render_header_start(atx_level: i32, raw_text: &str) -> Event<'static> {
    // Remove leading and trailing whitespace, convert to lowercase, and filter out punctuation.
    let text = raw_text
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_ascii_punctuation())
        .fold(String::new(), |mut acc, c| {
            acc.push(c);
            acc
        });

    if text.is_empty() {
        return Event::Start(Tag::Header(atx_level));
    }

    // Replace groups of spaces with dashes.
    let re_space_group = Regex::new(r"\s+").unwrap();
    let id = re_space_group.replace_all(&text, "-");

    Event::Html(format!("<h{} id=\"{}\">", atx_level, id).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod header_start {
        use super::*;

        #[test]
        fn empty_text() {
            assert_eq!(render_header_start(1, ""), Event::Start(Tag::Header(1)))
        }

        #[test]
        fn non_empty_text() {
            assert_eq!(
                render_header_start(1, "some text"),
                Event::Html("<h1 id=\"some-text\">".into())
            )
        }

        #[test]
        fn to_lowercase() {
            assert_eq!(
                render_header_start(1, "SoMe TeXt"),
                Event::Html("<h1 id=\"some-text\">".into())
            )
        }

        #[test]
        fn strip_whitespace() {
            assert_eq!(
                render_header_start(1, " some   text   "),
                Event::Html("<h1 id=\"some-text\">".into())
            )
        }

        #[test]
        fn removes_punctuation() {
            assert_eq!(
                render_header_start(1, "1. s'ome t:e;x`t"),
                Event::Html("<h1 id=\"1-some-text\">".into())
            )
        }
    }
}
