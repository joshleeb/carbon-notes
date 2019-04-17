use self::{
    code::{CodeBlock, SyntaxHighlighter},
    template::Template,
};
use crate::config::RenderConfig;
use pulldown_cmark::{html, Event, Parser, Tag};
use regex::Regex;
use std::{
    fs::File,
    io::{self, Read},
};

mod code;
mod template;

type ParserOptions = pulldown_cmark::Options;

#[derive(Default)]
struct RenderState {
    /// Title of the page if the first line is a header.
    title: Option<String>,
    /// ATX header level if a header is being processed.
    header: Option<i32>,
    /// Code block if a code block is being processed.
    code_block: Option<CodeBlock>,
}

/// Renders Markdown to HTML.
pub(crate) fn render(config: &RenderConfig, content: &str) -> io::Result<String> {
    let syntax_highlighter = SyntaxHighlighter::with_theme(&config.code_block_theme)?;
    let parser_opts = get_parser_opts();
    let md_parser =
        Parser::new_with_broken_link_callback(content, parser_opts, Some(&handle_broken_link));

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
                    if state.title.is_none() && atx_level == 1 {
                        state.title = Some(text.to_string());
                    }
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
                    events.push(syntax_highlighter.render(&code_block)?);
                    state.code_block = None;
                }
            }
            ev => events.push(ev),
        }
    }

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, events.into_iter());

    let mut tmpl = Template::new(&html_buf);
    if let Some(title) = state.title {
        tmpl.set_title(title);
    }

    // TODO: render::render support not inlining the stylesheet
    println!("{}", config.stylesheet_path.clone().unwrap().display());
    if let Some(ref path) = config.stylesheet_path {
        let mut stylesheet_content = String::new();
        File::open(path).and_then(|mut fh| fh.read_to_string(&mut stylesheet_content))?;
        tmpl.set_styles(stylesheet_content);
    }
    if config.mathjax_policy.inclusion() {
        tmpl.include_mathjax()
    }

    Ok(tmpl.to_string())
}

fn handle_broken_link(url: &str, title: &str) -> Option<(String, String)> {
    eprintln!("found broken link with: {}, {}", url, title);
    None
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

#[inline]
fn get_parser_opts() -> ParserOptions {
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(ParserOptions::ENABLE_TABLES);
    opts.insert(ParserOptions::ENABLE_FOOTNOTES);
    opts.insert(ParserOptions::ENABLE_STRIKETHROUGH);
    opts.insert(ParserOptions::ENABLE_TASKLISTS);
    opts
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
