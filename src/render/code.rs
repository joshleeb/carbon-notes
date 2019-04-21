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
            .find_syntax_by_token(&block.language_token)
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                format!("unknown syntax highlighting token {}", block.language_token),
            ))?;

        let html = highlighted_html_for_string(&block.code, &self.syntax_set, &syntax, &self.theme);
        Ok(Event::Html(CowStr::from(html)))
    }
}

pub(crate) struct CodeBlock {
    language_token: String,
    code: String,
}

impl Default for CodeBlock {
    fn default() -> Self {
        Self {
            language_token: "txt".into(),
            code: String::new(),
        }
    }
}

impl CodeBlock {
    pub(crate) fn with_language<S: ToString>(language_token: S) -> Self {
        let token = language_token.to_string();
        if token.is_empty() {
            return Self::default();
        }
        Self {
            language_token: token,
            code: String::new(),
        }
    }

    pub(crate) fn push(&mut self, code: &str) {
        self.code.push_str(code)
    }
}
