use crate::render::code::SyntaxHighlighter;
use std::io;

pub(crate) fn list_syntax_themes() -> io::Result<()> {
    SyntaxHighlighter::get_theme_names().iter().for_each(|tn| {
        println!("{}", tn);
    });
    Ok(())
}
