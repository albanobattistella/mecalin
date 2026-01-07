use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/main_action_list.ui")]
    pub struct MainActionList {
        #[template_child]
        pub action_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainActionList {
        const NAME: &'static str = "MainActionList";
        type Type = super::MainActionList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action("action.study-room", None, |obj, _, _| {
                obj.emit_by_name::<()>("study-room-selected", &[]);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainActionList {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_actions();
            self.setup_signals();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: std::sync::OnceLock<Vec<glib::subclass::Signal>> = std::sync::OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    glib::subclass::Signal::builder("study-room-selected").build(),
                    glib::subclass::Signal::builder("about-selected").build(),
                ]
            })
        }
    }
    impl WidgetImpl for MainActionList {}
    impl BoxImpl for MainActionList {}
}

impl imp::MainActionList {
    fn setup_actions(&self) {
        let actions = [
            ("Study room", "Learn typing fundamentals"),
            ("Student control", "Manage student progress"),
            ("Skill game", "Practice with games"),
            ("Videos", "Watch typing tutorials"),
            ("About", "Application information"),
        ];

        for (title, subtitle) in actions {
            let row = gtk::ListBoxRow::new();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            hbox.set_margin_top(12);
            hbox.set_margin_bottom(12);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);

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
            
            self.action_list.append(&row);
        }
    }

    fn setup_signals(&self) {
        let obj = self.obj().downgrade();
        self.action_list.connect_row_activated(move |_, row| {
            if let Some(obj) = obj.upgrade() {
                match row.index() {
                    0 => obj.emit_by_name::<()>("study-room-selected", &[]),
                    4 => obj.emit_by_name::<()>("about-selected", &[]),
                    _ => {}
                }
            }
        });
    }
}

glib::wrapper! {
    pub struct MainActionList(ObjectSubclass<imp::MainActionList>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MainActionList {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
