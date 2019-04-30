use crate::render::ToHtml;
use std::io;
use syntect::{
    highlighting::{Theme, ThemeSet},
    html::highlighted_html_for_string,
    parsing::{SyntaxReference, SyntaxSet},
};

const DEFAULT_LANGUAGE_TOKEN: &str = "txt";

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
}

pub(crate) struct CodeBlock<'a> {
    theme: &'a Theme,
    syntax_set: &'a SyntaxSet,
    syntax_ref: &'a SyntaxReference,
    code: String,
}

impl<'a> CodeBlock<'a> {
    pub(crate) fn new(highlighter: &'a SyntaxHighlighter, token: &str) -> io::Result<Self> {
        let language_token = match token.is_empty() {
            true => DEFAULT_LANGUAGE_TOKEN,
            _ => token,
        };
        highlighter
            .syntax_set
            .find_syntax_by_token(&language_token)
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                format!("unknown syntax highlighting token {}", language_token),
            ))
            .map(|syntax_ref| Self {
                theme: &highlighter.theme,
                syntax_set: &highlighter.syntax_set,
                syntax_ref,
                code: String::new(),
            })
    }

    pub(crate) fn push(&mut self, code: &str) {
        self.code.push_str(code)
    }
}

impl<'a> ToHtml for CodeBlock<'a> {
    fn to_html(&self) -> String {
        highlighted_html_for_string(&self.code, &self.syntax_set, &self.syntax_ref, &self.theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_language_token() {
        let highlighter = SyntaxHighlighter::with_theme("base16-ocean.dark").unwrap();
        assert!(CodeBlock::new(&highlighter, "invalid-language-token").is_err());
    }

    #[test]
    fn invalid_theme_name() {
        assert!(SyntaxHighlighter::with_theme("invalid-theme-name").is_err());
    }

    #[test]
    fn valid_language_token() {
        let highlighter = SyntaxHighlighter::with_theme("base16-ocean.dark").unwrap();
        let mut block = CodeBlock::new(&highlighter, "rs").unwrap();
        block.push("fn main() { println!(\"{}\"); }");

        let html = block.to_html();
        assert!(html.contains("fn") && html.contains("main"));
    }
}
