use render::RenderOptions;
use std::io::{self, Read, Write};

mod app;
mod render;

fn cmd_render(opts: RenderOptions) -> io::Result<()> {
    let stdin = io::stdin();
    let mut in_handle = stdin.lock();

    let mut md_content = String::new();
    let html_content = in_handle
        .read_to_string(&mut md_content)
        .and_then(|_| render::render(&opts, &md_content))?;

    let stdout = io::stdout();
    let mut out_handle = stdout.lock();
    write!(out_handle, "{}", html_content)
}

fn main() -> io::Result<()> {
    let matches = app::create().get_matches();

    match matches.subcommand() {
        ("render", Some(matches)) => cmd_render(matches.into()),
        _ => unimplemented!(),
    }
}
