use crate::render::{
    mathjax::{MathjaxPolicy, MATHJAX_CONFIG},
    stylesheet::Stylesheet,
};
use maud::{html, Markup, PreEscaped, DOCTYPE};

pub(crate) struct Template<'a> {
    pub content: Markup,
    pub title: &'a Option<String>,
    pub stylesheet: &'a Option<Stylesheet>,
    pub mathjax_policy: &'a MathjaxPolicy,
}

impl<'a> ToString for Template<'a> {
    fn to_string(&self) -> String {
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

// TODO: template::footer should have mathjax::MathjaxPolicy rendered by the maud::Rendered trait
fn footer(ctx: &Template) -> Markup {
    html! {
        footer {
            @if ctx.mathjax_policy.inclusion() {
                script type="text/x-mathjax-config" { (PreEscaped(MATHJAX_CONFIG)) }
                script type="text/javascript"
                    src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.5/MathJax.js" { }
            }
        }
    }
}
