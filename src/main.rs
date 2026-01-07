use gtk::prelude::*;
use libadwaita as adw;
use gio::prelude::*;

mod window;
use window::MecalinWindow;

fn main() {
    gio::resources_register_include!("resources.gresource")
        .expect("Failed to register resources");

    let app = adw::Application::builder()
        .application_id("com.example.mecalin")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = MecalinWindow::new(app);
    window.present();
}
