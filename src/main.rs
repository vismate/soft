use soft::app::App;

fn main() {
    let mut app = App::new().unwrap_or_else(|err| {
        eprintln!("{err}");
        panic!("app could not be inicialized")
    });

    app.load_or_default();
    app.run();
}
