#![feature(proc_macro_hygiene)]

use self::{
    app::{RenderArgs, SyncArgs},
    config::Config,
    stylesheet::Stylesheet,
};
use clap::ArgMatches;
use std::{
    convert::TryFrom,
    fs::File,
    io::{self, Read, Write},
};

mod app;
mod config;
mod info;
mod mathjax;
mod render;
mod stylesheet;
mod sync;

fn cmd_render(args: RenderArgs) -> io::Result<()> {
    let config = Config::default().render;

    let mut markdown = String::new();
    File::open(&args.input_path).and_then(|mut fh| fh.read_to_string(&mut markdown))?;

    let stylesheet = config
        .stylesheet_path
        .as_ref()
        .map(|path| Stylesheet::from_config(path, config.should_inline_stylesheet))
        .transpose()?;

    let rendered_html = render::render(
        &markdown,
        &stylesheet,
        &config.code_block_theme,
        &config.mathjax_policy,
    )?;
    File::create(&args.output_path).and_then(|mut fh| fh.write_all(&rendered_html.as_bytes()))
}

fn cmd_sync(_args: SyncArgs) -> io::Result<()> {
    let config = Config::default();
    sync::sync(&config)
}

fn cmd_info(matches: &ArgMatches<'static>) -> io::Result<()> {
    match matches.subcommand() {
        ("syntax-themes", _) => info::list_syntax_themes(),
        _ => unimplemented!(),
    }
}

fn main() -> io::Result<()> {
    let matches = app::create().get_matches();

    match matches.subcommand() {
        ("render", Some(matches)) => cmd_render(RenderArgs::try_from(matches)?),
        ("sync", Some(matches)) => cmd_sync(SyncArgs::try_from(matches)?),
        ("info", Some(matches)) => cmd_info(matches),
        _ => unimplemented!(),
    }
}
