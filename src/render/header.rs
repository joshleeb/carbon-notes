use crate::render::ToHtml;
use itertools::Itertools;
use regex::Regex;

pub(crate) struct HeaderStart<'a> {
    atx_level: i32,
    content: &'a str,
}

impl<'a> HeaderStart<'a> {
    pub(crate) fn new(atx_level: i32, content: &'a str) -> Self {
        Self { atx_level, content }
    }

    fn id(&self) -> String {
        let text = self
            .content
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .join("");

        // Replace groups of spaces with dashes.
        let re_space_group = Regex::new(r"\s+").unwrap();
        re_space_group.replace_all(&text, "-").to_string()
    }
}

impl<'a> ToHtml for HeaderStart<'a> {
    fn to_html(&self) -> String {
        let mut tag = format!("h{}", self.atx_level);
        if !self.content.trim().is_empty() {
            tag = format!("{} id=\"{}\"", tag, self.id());
        }
        format!("<{}>", tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text() {
        assert_eq!(HeaderStart::new(1, "").to_html(), "<h1>")
    }

    #[test]
    fn non_empty_text() {
        assert_eq!(
            HeaderStart::new(1, "some text").to_html(),
            "<h1 id=\"some-text\">"
        )
    }

    #[test]
    fn to_lowercase() {
        assert_eq!(
            HeaderStart::new(1, "SoMe TeXt").to_html(),
            "<h1 id=\"some-text\">"
        )
    }

    #[test]
    fn strip_whitespace() {
        assert_eq!(
            HeaderStart::new(1, " some   text   ").to_html(),
            "<h1 id=\"some-text\">"
        )
    }

    #[test]
    fn removes_punctuation() {
        assert_eq!(
            HeaderStart::new(1, "1. s'ome t:e;x`t").to_html(),
            "<h1 id=\"1-some-text\">"
        )
    }
}
