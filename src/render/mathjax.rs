use crate::render::ToHtml;
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

#[derive(Debug, PartialEq)]
pub(crate) enum MathjaxPolicy {
    Always,
    Never,
}

// TODO: MathjaxPolicy::inclusion implement the `Auto` policy
impl MathjaxPolicy {
    pub(crate) fn inclusion(&self) -> bool {
        *self == MathjaxPolicy::Always
    }
}

// TODO: MathjaxPolicy::render implement the option to load mathjax locally rather than from a CDN.
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

impl ToHtml for MathjaxPolicy {
    fn to_html(&self) -> String {
        self.render().into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclusion_always() {
        assert!(MathjaxPolicy::Always.inclusion())
    }

    #[test]
    fn inclusion_never() {
        assert!(!MathjaxPolicy::Never.inclusion())
    }

    #[test]
    fn render_template_with_config() {
        assert!(MathjaxPolicy::Always.to_html().contains("mathjax-config"))
    }

    #[test]
    fn render_empty_template() {
        assert!(MathjaxPolicy::Never.to_html().is_empty())
    }
}
