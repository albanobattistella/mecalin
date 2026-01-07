use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/study_room.ui")]
    pub struct StudyRoom {
        #[template_child]
        pub lesson_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StudyRoom {
        const NAME: &'static str = "StudyRoom";
        type Type = super::StudyRoom;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StudyRoom {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_room();
        }
    }
    impl WidgetImpl for StudyRoom {}
    impl BoxImpl for StudyRoom {}
}

impl imp::StudyRoom {
    fn setup_room(&self) {
        let lessons = [
            ("Start Course", "Begin or continue typing lessons"),
            ("Lesson Review", "Review previous lessons"),
            ("Speed Test", "Test typing speed and accuracy"),
            ("Practice Exercises", "Specific typing practice"),
            ("Student Report", "View progress report"),
        ];

        for (title, subtitle) in lessons {
            let row = gtk::ListBoxRow::new();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            hbox.set_margin_top(16);
            hbox.set_margin_bottom(16);
            hbox.set_margin_start(16);
            hbox.set_margin_end(16);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
            let title_label = gtk::Label::new(Some(title));
            title_label.set_halign(gtk::Align::Start);
            title_label.add_css_class("heading");
            
            let subtitle_label = gtk::Label::new(Some(subtitle));
            subtitle_label.set_halign(gtk::Align::Start);
            subtitle_label.add_css_class("dim-label");

            vbox.append(&title_label);
            vbox.append(&subtitle_label);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));
            
            self.lesson_list.append(&row);
        }
    }
}

glib::wrapper! {
    pub struct StudyRoom(ObjectSubclass<imp::StudyRoom>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl StudyRoom {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
