use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/text_view.ui")]
    pub struct TextView {}

    #[glib::object_subclass]
    impl ObjectSubclass for TextView {
        const NAME: &'static str = "MecalinTextView";
        type Type = super::TextView;
        type ParentType = gtk::TextView;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TextView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_monospace(true);
            self.obj().set_input_hints(gtk::InputHints::NONE);
        }
    }

    impl WidgetImpl for TextView {}
    impl TextViewImpl for TextView {}
}

glib::wrapper! {
    pub struct TextView(ObjectSubclass<imp::TextView>)
        @extends gtk::TextView, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

impl TextView {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for TextView {
    fn default() -> Self {
        Self::new()
    }
}
