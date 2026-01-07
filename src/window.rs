use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::subclass::prelude::*;

use crate::main_action_list::MainActionList;
use crate::study_room::StudyRoom;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/window.ui")]
    pub struct MecalinWindow {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub main_action_list_widget: TemplateChild<MainActionList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MecalinWindow {
        const NAME: &'static str = "MecalinWindow";
        type Type = super::MecalinWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            MainActionList::ensure_type();
            StudyRoom::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MecalinWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_signals();
        }
    }
    impl WidgetImpl for MecalinWindow {}
    impl WindowImpl for MecalinWindow {}
    impl ApplicationWindowImpl for MecalinWindow {}
    impl AdwApplicationWindowImpl for MecalinWindow {}
}

glib::wrapper! {
    pub struct MecalinWindow(ObjectSubclass<imp::MecalinWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MecalinWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn show_study_room(&self) {
        let imp = self.imp();
        imp.main_stack.set_visible_child_name("study_room");
    }
}

impl imp::MecalinWindow {
    fn setup_signals(&self) {
        let window = self.obj().downgrade();
        self.main_action_list_widget.connect_local("study-room-selected", false, move |_| {
            if let Some(window) = window.upgrade() {
                window.show_study_room();
            }
            None
        });
    }
}
