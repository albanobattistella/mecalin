use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::course::Lesson;
use crate::keyboard_widget::KeyboardWidget;
use crate::target_text_view::TargetTextView;
use crate::text_view::TextView;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/org/gnome/mecalin/ui/lesson_view.ui")]
    #[properties(wrapper_type = super::LessonView)]
    pub struct LessonView {
        #[template_child]
        pub lesson_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub target_text_view: TemplateChild<TargetTextView>,
        #[template_child]
        pub text_view: TemplateChild<TextView>,
        #[template_child]
        pub keyboard_container: TemplateChild<gtk::Box>,

        pub keyboard_widget: RefCell<Option<KeyboardWidget>>,
        #[property(get, set, nullable)]
        pub current_lesson: RefCell<Option<glib::BoxedAnyObject>>,
        #[property(get, set)]
        pub current_step_index: Cell<u32>,
        pub course: RefCell<Option<crate::course::Course>>,
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
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.setup_keyboard();
            self.setup_signals();
            self.setup_settings_signals();
        }
    }

    impl WidgetImpl for LessonView {}
    impl BoxImpl for LessonView {}
}

impl imp::LessonView {
    fn setup_keyboard(&self) {
        let keyboard = KeyboardWidget::new();
        self.keyboard_container.append(keyboard.widget());
        self.keyboard_widget.replace(Some(keyboard));
    }

    fn setup_signals(&self) {
        let keyboard_widget = self.keyboard_widget.borrow();
        if let Some(keyboard) = keyboard_widget.as_ref() {
            let keyboard_clone = keyboard.clone();
            let target_text_view = self.target_text_view.clone();
            let target_text_view_clone = self.target_text_view.clone();
            let lesson_view_clone = self.obj().downgrade();

            let buffer = self.text_view.text_view().buffer();
            buffer.connect_insert_text(move |buffer, _iter, text| {
                let current_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let target_buffer = target_text_view.text_view().buffer();
                let target_text = target_buffer.text(
                    &target_buffer.start_iter(),
                    &target_buffer.end_iter(),
                    false,
                );

                let current_str = current_text.as_str();
                let target_str = target_text.as_str();
                let new_text = format!("{}{}", current_str, text);

                // Check if the new text would match target text
                if !target_str.starts_with(&new_text) {
                    // Find the last space position or go to beginning
                    let last_space_pos = current_str.rfind(' ').map(|pos| pos + 1).unwrap_or(0);

                    // Reset to last space position
                    let corrected_text = &current_str[..last_space_pos];

                    glib::idle_add_local_once({
                        let buffer = buffer.clone();
                        let corrected_text = corrected_text.to_string();
                        move || {
                            buffer.set_text(&corrected_text);
                            let end_iter = buffer.end_iter();
                            buffer.place_cursor(&end_iter);
                        }
                    });
                }
            });

            buffer.connect_changed(move |buffer| {
                let typed_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let target_buffer = target_text_view_clone.text_view().buffer();
                let target_text = target_buffer.text(
                    &target_buffer.start_iter(),
                    &target_buffer.end_iter(),
                    false,
                );

                let typed_str = typed_text.as_str();
                let target_str = target_text.as_str();

                let cursor_pos = typed_str.chars().count() as i32;
                target_text_view_clone.set_cursor_position(cursor_pos);

                // Check if step is completed
                if typed_str == target_str && !target_str.is_empty() {
                    // Step completed - advance to next step or lesson
                    glib::idle_add_local_once({
                        let lesson_view = lesson_view_clone.clone();
                        move || {
                            if let Some(lesson_view) = lesson_view.upgrade() {
                                lesson_view.advance_to_next_step();
                            }
                        }
                    });
                    return;
                }

                // Update keyboard highlighting for next character
                let next_char = target_str.chars().nth(cursor_pos as usize);
                keyboard_clone.set_current_key(next_char);
            });
        }
    }

    fn setup_settings_signals(&self) {
        let obj = self.obj();
        obj.connect_notify_local(Some("current-step-index"), |lesson_view, _| {
            let settings = gio::Settings::new("org.gnome.mecalin");
            settings
                .set_uint("current-step", lesson_view.current_step_index() + 1)
                .unwrap();
        });
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
        self.set_current_lesson(Some(glib::BoxedAnyObject::new(lesson.clone())));

        // Save current lesson to settings
        let settings = gio::Settings::new("org.gnome.mecalin");
        settings.set_uint("current-lesson", lesson.id).unwrap();

        let imp = self.imp();
        imp.lesson_description.set_text(&lesson.description);

        // Reset step index
        self.set_current_step_index(0);

        // Set the first step's text as target text
        if let Some(first_step) = lesson.steps.first() {
            imp.target_text_view.set_text(&first_step.text);

            // Extract unique characters from the lesson text for keyboard display
            let mut target_keys = std::collections::HashSet::new();
            for ch in first_step.text.chars() {
                if ch.is_alphabetic() || ch == ' ' {
                    target_keys.insert(ch.to_lowercase().next().unwrap_or(ch));
                }
            }

            let keyboard_widget = imp.keyboard_widget.borrow();
            if let Some(keyboard) = keyboard_widget.as_ref() {
                keyboard.set_visible_keys(Some(target_keys));
            }
        }

        imp.text_view.set_text("");
    }

    pub fn set_course(&self, course: crate::course::Course) {
        let imp = self.imp();
        *imp.course.borrow_mut() = Some(course);
    }

    pub fn advance_to_next_step(&self) {
        let imp = self.imp();

        // Get the current lesson info without borrowing
        let (current_lesson_id, current_step, total_steps) = {
            let current_lesson_boxed = imp.current_lesson.borrow();
            if let Some(boxed) = current_lesson_boxed.as_ref() {
                if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                    (
                        lesson.id,
                        self.current_step_index() as usize,
                        lesson.steps.len(),
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let next_step = current_step + 1;

        if next_step < total_steps {
            // Move to next step within current lesson
            self.set_current_step_index(next_step as u32);

            let step_text = {
                let current_lesson_boxed = imp.current_lesson.borrow();
                if let Some(boxed) = current_lesson_boxed.as_ref() {
                    if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                        lesson.steps[next_step].text.clone()
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            imp.target_text_view.set_text(&step_text);
            imp.text_view.set_text("");

            // Update keyboard for new step
            let mut target_keys = std::collections::HashSet::new();
            for ch in step_text.chars() {
                if ch.is_alphabetic() || ch == ' ' {
                    target_keys.insert(ch.to_lowercase().next().unwrap_or(ch));
                }
            }

            let keyboard_widget = imp.keyboard_widget.borrow();
            if let Some(keyboard) = keyboard_widget.as_ref() {
                keyboard.set_visible_keys(Some(target_keys));
            }
        } else {
            // Current lesson completed - try to load next lesson
            let next_lesson_option = {
                let course = imp.course.borrow();
                course
                    .as_ref()
                    .and_then(|c| c.get_lesson(current_lesson_id + 1).cloned())
            };

            if let Some(next_lesson) = next_lesson_option {
                // Load next lesson
                self.set_lesson(&next_lesson);
            } else {
                // Check if we have a course to determine the message
                let has_course = imp.course.borrow().is_some();
                if has_course {
                    // All lessons completed
                    imp.target_text_view
                        .set_text("Course completed! Congratulations!");
                } else {
                    // No course set, just show lesson completion
                    imp.target_text_view
                        .set_text("Lesson completed! Well done!");
                }
                imp.text_view.set_text("");
            }
        }
    }
}
