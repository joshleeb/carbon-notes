use crate::render::MathjaxPolicy;
use clap::{crate_authors, crate_version, App, Arg, SubCommand};

pub(crate) fn create() -> App<'static, 'static> {
    App::new("carbon")
        .version(crate_version!())
        .author(crate_authors!())
        .version_short("v")
        .subcommand(
            SubCommand::with_name("render")
                .about("Render a Markdown document")
                .arg(
                    Arg::with_name("stylesheet")
                        .long("stylesheet")
                        .required(false)
                        .takes_value(true)
                        .help("Stylesheet to be inlined"),
                )
                .arg(
                    Arg::with_name("syntax-theme")
                        .long("syntax-theme")
                        .required(false)
                        .takes_value(true)
                        .help("Syntax highlighting theme"),
                )
                .arg(
                    Arg::with_name("mathjax-policy")
                        .long("mathjax")
                        .case_insensitive(true)
                        .required(false)
                        .takes_value(true)
                        .possible_values(&MathjaxPolicy::variants())
                        .help("Policy for including script tags for loading Mathjax from CDN"),
                ),
        )
        .subcommand(SubCommand::with_name("sync").about("in progress..."))
        .subcommand(
            SubCommand::with_name("info")
                .about("Display useful information")
                .subcommand(
                    SubCommand::with_name("syntax-themes")
                        .about("Display the list of known syntax highlighting themes"),
                ),
        )
}
