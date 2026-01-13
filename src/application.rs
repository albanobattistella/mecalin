use gio::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::subclass::prelude::*;

use crate::window::MecalinWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MecalinApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for MecalinApplication {
        const NAME: &'static str = "MecalinApplication";
        type Type = super::MecalinApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MecalinApplication {}
    impl ApplicationImpl for MecalinApplication {
        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.set_resource_base_path(Some("/io/github/nacho/mecalin"));
        }

        fn activate(&self) {
            let app = self.obj();
            let window = MecalinWindow::new(app.upcast_ref());
            window.load_window_state();
            window.present();
        }
    }
    impl GtkApplicationImpl for MecalinApplication {}
    impl AdwApplicationImpl for MecalinApplication {}
}

glib::wrapper! {
    pub struct MecalinApplication(ObjectSubclass<imp::MecalinApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MecalinApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "io.github.nacho.mecalin")
            .build()
    }
}
