use crate::render::{mathjax::MathjaxPolicy, stylesheet::Stylesheet, ToHtml};
use maud::{html, Markup, DOCTYPE};

pub(crate) struct Template<'a> {
    pub content: Markup,
    pub title: &'a Option<String>,
    pub stylesheet: &'a Option<Stylesheet>,
    pub mathjax_policy: &'a MathjaxPolicy,
}

impl<'a> ToHtml for Template<'a> {
    fn to_html(&self) -> String {
        page(&self).into_string()
    }
}

fn page(ctx: &Template) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (head(ctx))
            body {
                (ctx.content)
                (footer(ctx))
            }
        }
    }
}

fn head(ctx: &Template) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            @if let Some(ref title) = ctx.title {
                title { (title) }
            }
            @if let Some(ref stylesheet) = ctx.stylesheet {
                (stylesheet)
            }
        }
    }
}

fn footer(ctx: &Template) -> Markup {
    html! {
        footer { (ctx.mathjax_policy) }
    }
}
