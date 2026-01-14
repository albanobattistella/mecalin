use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::course::Course;
use crate::lesson_view::LessonView;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/study_room.ui")]
    pub struct StudyRoom {
        #[template_child]
        pub lesson_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub lesson_view_widget: TemplateChild<LessonView>,

        pub course: std::cell::RefCell<Option<Course>>,
        pub settings: std::cell::RefCell<Option<gio::Settings>>,
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
            self.setup_signals();

            // Automatically show lesson view
            self.obj().show_lesson_view();
        }
    }

    impl WidgetImpl for StudyRoom {}
    impl BoxImpl for StudyRoom {}
}

impl imp::StudyRoom {
    fn setup_room(&self) {
        let language = crate::utils::language_from_locale();
        let course = Course::new_with_language(language).unwrap_or_default();
        self.course.replace(Some(course));

        let settings = gio::Settings::new("io.github.nacho.mecalin");
        self.settings.replace(Some(settings));

        let menu_items = [
            (
                gettext("Start Course"),
                gettext("Begin or continue typing lessons"),
            ),
            // TODO: (gettext("Lesson Review"), gettext("Review previous lessons")),
            // TODO: (
            //     gettext("Speed Test"),
            //     gettext("Test typing speed and accuracy"),
            // ),
            // TODO: (
            //     gettext("Practice Exercises"),
            //     gettext("Specific typing practice"),
            // ),
            // TODO: (gettext("Student Report"), gettext("View progress report")),
        ];

        for (title, subtitle) in menu_items {
            let row = gtk::ListBoxRow::new();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            hbox.set_margin_top(16);
            hbox.set_margin_bottom(16);
            hbox.set_margin_start(16);
            hbox.set_margin_end(16);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
            let title_label = gtk::Label::new(Some(&title));
            title_label.set_halign(gtk::Align::Start);
            title_label.add_css_class("heading");

            let subtitle_label = gtk::Label::new(Some(&subtitle));
            subtitle_label.set_halign(gtk::Align::Start);
            subtitle_label.add_css_class("dim-label");

            vbox.append(&title_label);
            vbox.append(&subtitle_label);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));

            self.lesson_list.append(&row);
        }
    }

    fn setup_signals(&self) {
        let obj = self.obj().downgrade();
        self.lesson_list.connect_row_activated(move |_, row| {
            if row.index() == 0 {
                if let Some(study_room) = obj.upgrade() {
                    study_room.show_lesson_view();
                }
            }
        });
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

    pub fn can_go_back(&self) -> bool {
        let imp = self.imp();
        imp.main_stack.visible_child_name().as_deref() == Some("lesson_view")
    }

    pub fn go_back(&self) {
        let imp = self.imp();
        imp.main_stack.set_visible_child_name("lesson_list");

        // Clear current lesson to reset window title
        imp.lesson_view_widget
            .set_current_lesson(None::<glib::BoxedAnyObject>);
    }

    pub fn show_lesson_view(&self) {
        let imp = self.imp();
        if let Some(course) = imp.course.borrow().as_ref() {
            let settings = gio::Settings::new("io.github.nacho.mecalin");
            let current_lesson = settings.uint("current-lesson");
            let current_step = settings.uint("current-step");

            if let Some(lesson) = course.get_lesson(current_lesson) {
                imp.lesson_view_widget.set_course(course.clone());
                imp.lesson_view_widget.set_lesson(lesson);

                // Load the correct step after setting the lesson
                let step_index = if current_step > 0 {
                    current_step - 1
                } else {
                    0
                };
                imp.lesson_view_widget.load_step(step_index);

                imp.main_stack.set_visible_child_name("lesson_view");
            }
        }
    }

    pub fn lesson_view(&self) -> &LessonView {
        &self.imp().lesson_view_widget
    }
}
