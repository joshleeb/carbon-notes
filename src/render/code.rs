use pulldown_cmark::{CowStr, Event};
use std::io;
use syntect::{
    highlighting::{Theme, ThemeSet},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
};

pub(crate) struct SyntaxHighlighter {
    theme: Theme,
    syntax_set: SyntaxSet,
}

impl SyntaxHighlighter {
    pub(crate) fn with_theme(theme_name: &str) -> io::Result<Self> {
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes.get(theme_name).ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            format!("unknown syntax highlighting theme {}", theme_name),
        ))?;

        Ok(Self {
            theme: theme.clone(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
        })
    }

    pub(crate) fn render(&self, block: &CodeBlock) -> io::Result<Event> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(&block.lang)
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                format!("unknown syntax highlighting token {}", block.lang),
            ))?;

        let html = highlighted_html_for_string(&block.code, &self.syntax_set, &syntax, &self.theme);
        Ok(Event::Html(CowStr::from(html)))
    }
}

pub(crate) struct CodeBlock {
    lang: String,
    code: String,
}

impl CodeBlock {
    pub(crate) fn new<S: ToString>(lang: S) -> Self {
        Self {
            lang: lang.to_string(),
            code: String::new(),
        }
    }

    pub(crate) fn push(&mut self, code: &str) {
        self.code.push_str(code)
    }
}
