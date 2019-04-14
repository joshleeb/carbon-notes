use maud::{html, Markup, PreEscaped, DOCTYPE};

pub(crate) struct Template<'a> {
    ctx: Context<'a>,
}

impl<'a> Template<'a> {
    pub(crate) fn new(title: Option<String>, content: &'a str) -> Self {
        let ctx = Context { title, content };
        Self { ctx }
    }
}

impl<'a> ToString for Template<'a> {
    fn to_string(&self) -> String {
        page(&self.ctx).into_string()
    }
}

// TODO: template::Context
//  - Shouldn't need to move or copy anything here as we are just reading values
struct Context<'a> {
    title: Option<String>,
    content: &'a str,
}

fn page(ctx: &Context) -> Markup {
    html! {
        (header(ctx))
        body { (PreEscaped(ctx.content)) }
        (footer())
    }
}

fn header(ctx: &Context) -> Markup {
    html! {
        (DOCTYPE)
        html {
            meta charset="utf-8";
            @if let Some(ref title) = ctx.title {
                title { (title) }
            }
        }
    }
}

fn footer() -> Markup {
    html! {
        footer { }
    }
}
