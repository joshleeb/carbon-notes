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
    // Never,
}

// TODO: config::MathjaxPolicy implement the `Auto` policy
impl MathjaxPolicy {
    pub(crate) fn inclusion(&self) -> bool {
        *self == MathjaxPolicy::Always
    }
}
