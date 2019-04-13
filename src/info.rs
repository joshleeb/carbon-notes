use crate::render::code::SyntaxHighlighter;
use std::io;

pub(crate) fn list_syntax_themes() -> io::Result<()> {
    let highlighter = SyntaxHighlighter::default();
    highlighter.get_theme_names().iter().for_each(|tn| {
        println!("{}", tn);
    });
    Ok(())
}
