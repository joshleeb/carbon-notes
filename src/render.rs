use self::{
    code::{CodeBlock, SyntaxHighlighter},
    header::HeaderStart,
    mathjax::MathjaxPolicy,
    note::Note,
    stylesheet::Stylesheet,
};
use pulldown_cmark::{html, Event, Parser, Tag};
use std::io;

pub mod code;
pub mod index;
pub mod mathjax;
pub mod note;
pub mod stylesheet;

mod header;
mod template;

type ParserOptions = pulldown_cmark::Options;

pub trait ToHtml {
    fn to_html(&self) -> String;
}

#[derive(Default)]
struct RenderState<'a> {
    /// Title of the page if the first line is a header.
    title: Option<String>,
    /// ATX header level if a header is being processed.
    header: Option<i32>,
    /// Code block if a code block is being processed.
    code_block: Option<CodeBlock<'a>>,
}

pub struct RenderOpts<'a> {
    stylesheet: &'a Option<Stylesheet>,
    syntax_highlighter: &'a SyntaxHighlighter,
    mathjax_policy: &'a MathjaxPolicy,
}

impl<'a> RenderOpts<'a> {
    pub fn new(
        stylesheet: &'a Option<Stylesheet>,
        syntax_highlighter: &'a SyntaxHighlighter,
        mathjax_policy: &'a MathjaxPolicy,
    ) -> Self {
        Self {
            stylesheet,
            syntax_highlighter,
            mathjax_policy,
        }
    }

    pub fn render(&self, markdown: &str) -> io::Result<String> {
        let md_parser = self.md_parser(&markdown);
        let (state, events) = self.process_events(md_parser)?;

        let mut html_buf = String::new();
        html::push_html(&mut html_buf, events.into_iter());

        Ok(Note::new(
            &html_buf,
            &state.title,
            self.stylesheet,
            self.mathjax_policy,
        )
        .to_html())
    }

    fn process_events(
        &self,
        events: impl Iterator<Item = Event<'a>>,
    ) -> io::Result<(RenderState, Vec<Event>)> {
        let mut state = RenderState::default();
        let mut processed_events = vec![];

        for event in events {
            match event {
                Event::Start(Tag::Header(atx_level)) => {
                    state.header = Some(atx_level);
                }
                Event::Start(Tag::CodeBlock(language)) => {
                    state.code_block = Some(CodeBlock::new(&self.syntax_highlighter, &language)?);
                }
                Event::Text(text) => {
                    state.header = state.header.and_then(|atx_level| {
                        if state.title.is_none() && atx_level == 1 {
                            state.title = Some(text.to_string());
                        }
                        let header_start = HeaderStart::new(atx_level, &text);
                        processed_events.push(Event::Html(header_start.to_html().into()));
                        None
                    });
                    if let Some(ref mut code_block) = state.code_block {
                        code_block.push(&text);
                        continue;
                    }
                    processed_events.push(Event::Text(text));
                }
                Event::End(Tag::CodeBlock(_)) => {
                    state.code_block = state.code_block.and_then(|block| {
                        processed_events.push(Event::Html(block.to_html().into()));
                        None
                    });
                }
                ev => processed_events.push(ev),
            }
        }
        Ok((state, processed_events))
    }

    #[inline]
    fn md_parser(&self, content: &'a str) -> Parser<'a> {
        let mut opts = pulldown_cmark::Options::empty();
        opts.insert(ParserOptions::ENABLE_TABLES);
        opts.insert(ParserOptions::ENABLE_FOOTNOTES);
        opts.insert(ParserOptions::ENABLE_STRIKETHROUGH);
        opts.insert(ParserOptions::ENABLE_TASKLISTS);

        // TODO: RenderOpts::md_parser should include broken link callback
        Parser::new_with_broken_link_callback(content, opts, None)
    }
}
