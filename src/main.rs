use rusticrayz::Application;

fn main() {
    tracing_subscriber::fmt::init();

    let mut app = Application::new("Rusticrays");
    app.run();
}
