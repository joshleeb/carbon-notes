use maud::{html, Markup, PreEscaped, DOCTYPE};

const MATHJAX_CONFIG: &str = "
    MathJax.Hub.Config({
        extensions: [\"tex2jax.js\"],
        jax: [\"input/TeX\", \"output/HTML-CSS\"],
        tex2jax: {
          inlineMath: [['$','$']],
          displayMath: [['$$','$$']],
          processEscapes: true
        },
        \"HTML-CSS\": { fonts: [\"TeX\"] }
    });
";

pub(crate) struct Template<'a> {
    ctx: Context<'a>,
}

impl<'a> Template<'a> {
    pub(crate) fn new(content: &'a str) -> Self {
        let ctx = Context {
            content,
            ..Default::default()
        };
        Self { ctx }
    }

    pub(crate) fn set_title(&mut self, title: String) {
        self.ctx.title = Some(title);
    }

    pub(crate) fn set_styles(&mut self, styles: String) {
        self.ctx.styles = Some(styles);
    }

    pub(crate) fn include_mathjax(&mut self) {
        self.ctx.include_mathjax = true;
    }
}

impl<'a> ToString for Template<'a> {
    fn to_string(&self) -> String {
        page(&self.ctx).into_string()
    }
}

// TODO: template::Context
//  - Shouldn't need to move or copy anything here as we are just reading values
#[derive(Default)]
struct Context<'a> {
    content: &'a str,
    title: Option<String>,
    styles: Option<String>,
    include_mathjax: bool,
}

fn page(ctx: &Context) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (head(ctx))
            body {
                (PreEscaped(ctx.content))
                (footer(ctx))
            }
        }
    }
}

fn head(ctx: &Context) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            @if let Some(ref title) = ctx.title {
                title { (title) }
            }
            @if let Some(ref styles) = ctx.styles {
                style { (PreEscaped(styles)) }
            }
        }
    }
}

fn footer(ctx: &Context) -> Markup {
    html! {
        footer {
            @if ctx.include_mathjax {
                script type="text/x-mathjax-config" { (PreEscaped(MATHJAX_CONFIG)) }
                script type="text/javascript"
                    src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.5/MathJax.js" { }
            }
        }
    }
}
