use pulldown_cmark::{CowStr, Event};
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

pub(crate) struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

// TODO: Implement SyntaxHighlighter::with_theme
impl SyntaxHighlighter {
    // TODO: Handle errors in SyntaxHighlighter::render
    pub(crate) fn render(&self, block: &CodeBlock) -> Event {
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let syntax = self.syntax_set.find_syntax_by_token(&block.lang).unwrap();
        let html = highlighted_html_for_string(&block.code, &self.syntax_set, &syntax, theme);
        Event::Html(CowStr::from(html))
    }

    pub(crate) fn get_theme_names(&self) -> Vec<String> {
        let keys = self.theme_set.themes.keys();
        keys.map(|k| k.to_string()).collect()
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
