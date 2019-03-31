use clap::{arg_enum, crate_authors, crate_version, App, Arg, SubCommand};

#[derive(Debug, PartialEq)]
enum MathjaxPolicy {
    Auto,
    Always,
    Never,
}

impl MathjaxPolicy {
    // TODO: Generate with proc-macro.
    fn variants() -> &'static [&'static str] {
        &["auto", "always", "never"]
    }
}

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
                        .help("Stylesheet to include"),
                )
                .arg(
                    Arg::with_name("inline-style")
                        .long("inline-style")
                        .required(false)
                        .takes_value(false)
                        .help("Whether to inline stylesheet in rendered HTML"),
                )
                .arg(
                    Arg::with_name("mathjax-policy")
                        .long("mathjax")
                        .case_insensitive(true)
                        .required(false)
                        .default_value("auto")
                        .possible_values(&MathjaxPolicy::variants())
                        .help("Policy for including script tags for loading Mathjax from CDN"),
                ),
        )
        .subcommand(SubCommand::with_name("sync").about("in progress..."))
}
