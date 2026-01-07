use gio::prelude::*;

mod application;
mod config;
mod course;
mod keyboard_widget;
mod lesson_view;
mod main_action_list;
mod study_room;
mod target_text_view;
mod text_view;
mod window;

use application::MecalinApplication;

fn main() {
    gio::resources_register_include!("resources.gresource").expect("Failed to register resources");

    let app = MecalinApplication::new();
    app.run();
}
