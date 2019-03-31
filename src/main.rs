mod app;

fn main() {
    let matches = app::create().get_matches();
    println!("{:?}", matches);
}
