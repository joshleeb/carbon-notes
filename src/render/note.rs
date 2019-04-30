use crate::render::{mathjax::MathjaxPolicy, stylesheet::Stylesheet, template::Template, ToHtml};
use maud::{html, Markup, PreEscaped, Render};

pub(crate) struct Note<'a> {
    rendered_html: &'a str,
    title: &'a Option<String>,
    stylesheet: &'a Option<Stylesheet>,
    mathjax_policy: &'a MathjaxPolicy,
}

impl<'a> Note<'a> {
    pub(crate) fn new(
        rendered_html: &'a str,
        title: &'a Option<String>,
        stylesheet: &'a Option<Stylesheet>,
        mathjax_policy: &'a MathjaxPolicy,
    ) -> Self {
        Self {
            rendered_html,
            title,
            stylesheet,
            mathjax_policy,
        }
    }
}

impl<'a> Render for Note<'a> {
    fn render(&self) -> Markup {
        html! { (PreEscaped(self.rendered_html)) }
    }
}

impl<'a> ToHtml for Note<'a> {
    fn to_html(&self) -> String {
        Template {
            content: self.render(),
            title: self.title,
            stylesheet: self.stylesheet,
            mathjax_policy: self.mathjax_policy,
        }
        .to_html()
    }
}
