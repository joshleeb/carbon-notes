pub(crate) use mathjax::MathjaxPolicy;

use clap::ArgMatches;
use std::path::PathBuf;

mod mathjax;

#[derive(Debug)]
pub(crate) struct RenderOptions {
    stylesheet_path: Option<PathBuf>,
    should_inline_style: bool,
    mathjax_policy: MathjaxPolicy,
}

impl From<&ArgMatches<'static>> for RenderOptions {
    fn from(matches: &ArgMatches<'static>) -> Self {
        let stylesheet_path = matches.value_of("stylesheet").map(PathBuf::from);
        let mathjax_policy = matches.value_of("mathjax-policy").unwrap_or("auto");

        Self {
            stylesheet_path,
            should_inline_style: matches.is_present("inline-style"),
            mathjax_policy: mathjax_policy.parse().unwrap(),
        }
    }
}
