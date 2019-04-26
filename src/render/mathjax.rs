use maud::{html, Markup, PreEscaped, Render};

pub(crate) const MATHJAX_CONFIG: &str = "
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

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum MathjaxPolicy {
    Always,
    Never,
}

// TODO: config::MathjaxPolicy implement the `Auto` policy
impl MathjaxPolicy {
    pub(crate) fn inclusion(&self) -> bool {
        *self == MathjaxPolicy::Always
    }
}

impl Render for MathjaxPolicy {
    fn render(&self) -> Markup {
        html! {
            @if self.inclusion() {
                script type="text/x-mathjax-config" { (PreEscaped(MATHJAX_CONFIG)) }
                script type="text/javascript"
                    src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.5/MathJax.js" { }
            }
        }
    }
}
