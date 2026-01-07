use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/example/mecalin/ui/window.ui")]
    pub struct MecalinWindow {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub main_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub welcome_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MecalinWindow {
        const NAME: &'static str = "MecalinWindow";
        type Type = super::MecalinWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MecalinWindow {}
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
}
