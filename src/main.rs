use render::RenderOptions;

mod app;
mod render;

fn render(opts: RenderOptions) {
    println!("{:?}", opts);
}

fn main() {
    let matches = app::create().get_matches();

    match matches.subcommand() {
        ("render", Some(matches)) => render(matches.into()),
        _ => unimplemented!(),
    }
}
