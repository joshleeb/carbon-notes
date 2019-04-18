#![feature(proc_macro_hygiene)]

use self::{app::Args, config::Config};
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

fn cmd_render(args: Args) -> io::Result<()> {
    let config = Config::read_or_write_default(&args.config_path)?;

    let mut md_content = String::new();
    File::open(&args.input_path).and_then(|mut fh| fh.read_to_string(&mut md_content))?;

    let html_content = render::render(&config.render, &md_content)?;
    File::create(&args.output_path).and_then(|mut fh| fh.write_all(&html_content.as_bytes()))
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
        ("render", Some(matches)) => cmd_render(Args::try_from(matches)?),
        ("info", Some(matches)) => cmd_info(matches),
        _ => unimplemented!(),
    }
}
