use std::io;
use syntect::highlighting::ThemeSet;

pub fn list_syntax_themes() -> io::Result<()> {
    let theme_set = ThemeSet::load_defaults();
    let keys = theme_set.themes.keys();
    keys.map(ToString::to_string).for_each(|tn| {
        println!("{}", tn);
    });
    Ok(())
}
