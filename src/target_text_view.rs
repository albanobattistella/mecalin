use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/target_text_view.ui")]
    pub struct TargetTextView {
        #[template_child]
        pub text_view: TemplateChild<gtk::TextView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TargetTextView {
        const NAME: &'static str = "MecalinTargetTextView";
        type Type = super::TargetTextView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TargetTextView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_text_view();
        }
    }
    impl WidgetImpl for TargetTextView {}
    impl BoxImpl for TargetTextView {}
}

impl imp::TargetTextView {
    fn setup_text_view(&self) {
        // Prevent the target text view from being focused
        self.text_view.set_can_focus(false);
        // Ensure cursor remains visible even when not focused
        self.text_view.set_cursor_visible(true);
    }
}

glib::wrapper! {
    pub struct TargetTextView(ObjectSubclass<imp::TargetTextView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl TargetTextView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn text_view(&self) -> &gtk::TextView {
        &self.imp().text_view
    }

    pub fn set_text(&self, text: &str) {
        let buffer = self.text_view().buffer();
        buffer.set_text(text);
        buffer.place_cursor(&buffer.start_iter());
    }

    pub fn set_cursor_position(&self, position: i32) {
        let buffer = self.text_view().buffer();
        let mut iter = buffer.start_iter();
        iter.forward_chars(position);
        buffer.place_cursor(&iter);
    }
}

impl Default for TargetTextView {
    fn default() -> Self {
        Self::new()
    }
}
