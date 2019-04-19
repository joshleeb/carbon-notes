use clap::{crate_authors, crate_version, App, Arg, ArgMatches, SubCommand};
use std::{
    convert::TryFrom,
    io,
    path::{Path, PathBuf},
};

pub(crate) fn create() -> App<'static, 'static> {
    App::new("carbon")
        .version(crate_version!())
        .author(crate_authors!())
        .version_short("v")
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .required(false)
                .takes_value(true)
                .help("Configuration file to use"),
        )
        .subcommand(
            SubCommand::with_name("render")
                .about("Render a Markdown document")
                .arg(arg_config())
                // TODO app::create render `file` arg should be optional. Then we can read from
                // stdin if a file path is not provided.
                .arg(
                    Arg::with_name("FILE")
                        .required(true)
                        .index(1)
                        .help("Markdown file to render"),
                )
                // TODO app::create render `output` arg should be optional. Then we can write to
                // stdout if a path is not provided.
                .arg(
                    Arg::with_name("output")
                        .long("output")
                        .short("o")
                        .required(false)
                        .takes_value(true)
                        .help("Path to output rendered HTML"),
                )
                .arg(
                    Arg::with_name("force")
                        .long("force")
                        .short("f")
                        .required(false)
                        .takes_value(false)
                        .help("Force output file to overwrite existing files"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sync")
                // TODO: app::sync better about message
                .about("Sync a directory to it's rendered equivalent"),
        )
        .subcommand(
            SubCommand::with_name("info")
                .about("Display useful information")
                .subcommand(
                    SubCommand::with_name("syntax-themes")
                        .about("Display the list of known syntax highlighting themes"),
                ),
        )
}

#[inline]
fn arg_config() -> Arg<'static, 'static> {
    Arg::with_name("config")
        .long("config")
        .short("c")
        .required(false)
        .takes_value(true)
        .help("Configuration file to use")
}

#[derive(Debug)]
pub(crate) struct Args {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub overwrite_output: bool,
    pub config_path: PathBuf,
}

impl TryFrom<&ArgMatches<'static>> for Args {
    type Error = io::Error;

    fn try_from(matches: &ArgMatches<'static>) -> Result<Self, Self::Error> {
        let input_path = matches.value_of("FILE").map(PathBuf::from).unwrap();
        let config_path = get_config_path(matches.value_of("config").map(PathBuf::from))?;
        let overwrite_output = matches.is_present("force");
        let output_path = get_output_path(
            matches.value_of("output").map(PathBuf::from),
            &input_path,
            overwrite_output,
        )?;

        Ok(Self {
            input_path,
            output_path,
            overwrite_output,
            config_path,
        })
    }
}

// TODO: app::get_output_path refactor to use option/result composition functions
//  - Should remove duplicate code of checking foce and if the path exists.
//  - Also add tests.
fn get_output_path(output: Option<PathBuf>, input: &Path, overwrite: bool) -> io::Result<PathBuf> {
    if let Some(path) = output {
        if !overwrite && path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("file exists at output path {}", path.display()),
            ));
        }
        return Ok(path);
    }

    let output_path = input.with_extension("html");
    if !overwrite && output_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("file exists at output path {}", output_path.display()),
        ));
    }
    Ok(output_path)
}

// TODO: Refactor app::get_config_path to use Option and Result composition functions.
//  - Also add tests.
fn get_config_path(path: Option<PathBuf>) -> io::Result<PathBuf> {
    if let Some(config_path) = path {
        return Ok(config_path);
    }

    dirs::config_dir()
        .map(|base| base.join("carbon/config.toml"))
        .ok_or(io::Error::new(
            io::ErrorKind::Other,
            "unable to determine config directory",
        ))
}
