use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::course::Lesson;
use crate::keyboard_widget::KeyboardWidget;
use crate::text_view::TextView;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/lesson_view.ui")]
    pub struct LessonView {
        #[template_child]
        pub lesson_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub lesson_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub target_text_view: TemplateChild<gtk::TextView>,
        #[template_child]
        pub text_view: TemplateChild<TextView>,
        #[template_child]
        pub keyboard_container: TemplateChild<gtk::Box>,

        pub keyboard_widget: std::cell::RefCell<Option<KeyboardWidget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LessonView {
        const NAME: &'static str = "LessonView";
        type Type = super::LessonView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LessonView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_keyboard();
            self.setup_signals();
            self.setup_target_text_view();
        }
    }
    impl WidgetImpl for LessonView {}
    impl BoxImpl for LessonView {}
}

impl imp::LessonView {
    fn setup_keyboard(&self) {
        let keyboard = KeyboardWidget::new();
        self.keyboard_container.append(keyboard.widget());
        *self.keyboard_widget.borrow_mut() = Some(keyboard);
    }

    fn setup_signals(&self) {
        let keyboard_widget = self.keyboard_widget.borrow();
        if let Some(keyboard) = keyboard_widget.as_ref() {
            let keyboard_clone = keyboard.clone();
            let target_text_view = self.target_text_view.clone();

            let buffer = self.text_view.text_view().buffer();
            buffer.connect_changed(move |buffer| {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let cursor_pos = text.chars().count();

                // Update keyboard highlighting
                if let Some(last_char) = text.chars().last() {
                    keyboard_clone.set_current_key(Some(last_char));
                } else {
                    keyboard_clone.set_current_key(None);
                }

                // Update cursor position in target text view
                let target_buffer = target_text_view.buffer();
                let mut iter = target_buffer.start_iter();
                iter.forward_chars(cursor_pos as i32);
                target_buffer.place_cursor(&iter);
            });
        }
    }

    fn setup_target_text_view(&self) {
        self.target_text_view.set_can_focus(false);
        self.target_text_view.set_can_target(false);
        self.target_text_view.set_editable(false);
        self.target_text_view.set_monospace(true);

        // Ensure cursor remains visible even when not focused
        self.target_text_view.set_cursor_visible(true);
    }
}

glib::wrapper! {
    pub struct LessonView(ObjectSubclass<imp::LessonView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl LessonView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_lesson(&self, lesson: &Lesson) {
        let imp = self.imp();
        let title = format!("{} {}", gettext("Lesson"), lesson.id);
        imp.lesson_title.set_text(&title);
        imp.lesson_description.set_text(&lesson.description);

        // Set the first step's text as target text
        if let Some(first_step) = lesson.steps.first() {
            let target_buffer = imp.target_text_view.buffer();
            target_buffer.set_text(&first_step.text);
            // Place cursor at the beginning
            target_buffer.place_cursor(&target_buffer.start_iter());
        }

        imp.text_view.set_text("");
    }
}
