use gio::prelude::*;

mod application;
mod main_action_list;
mod study_room;
mod window;

use application::MecalinApplication;

fn main() {
    gio::resources_register_include!("resources.gresource")
        .expect("Failed to register resources");

    let app = MecalinApplication::new();
    app.run();
}
