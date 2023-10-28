use rusticrayz::Application;

fn main() {
    tracing_subscriber::fmt::init();

    let app = Application::new("Rusticrays");
    app.run();
}
