#![feature(proc_macro_hygiene)]

use self::{
    app::{RenderArgs, SyncArgs},
    config::Config,
    render::{stylesheet::Stylesheet, RenderOpts},
    sync::SyncOpts,
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
mod render;
mod sync;

fn cmd_render(args: RenderArgs) -> io::Result<()> {
    let config = Config::default();

    let mut markdown = String::new();
    File::open(&args.input_path).and_then(|mut fh| fh.read_to_string(&mut markdown))?;

    let stylesheet_path = config.render.stylesheet_path.as_ref();
    let stylesheet = stylesheet_path
        .map(|path| Stylesheet::new(path, config.render.should_inline_stylesheet))
        .transpose()?;

    let render = RenderOpts::new(
        &stylesheet,
        &config.render.code_block_theme,
        &config.render.mathjax_policy,
    );
    let html = render.render(&markdown)?;
    File::create(&args.output_path).and_then(|mut fh| fh.write_all(&html.as_bytes()))
}

fn cmd_sync(_args: SyncArgs) -> io::Result<()> {
    let config = Config::default();
    SyncOpts::try_from(config)?.sync()
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
