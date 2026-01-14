use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, DrawingArea};
use i18n_format::i18n_fmt;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

const KEYS: &[char] = &[
    'a', 's', 'd', 'f', 'j', 'k', 'l', 'q', 'w', 'e', 'r', 'u', 'i', 'o', 'p',
];

#[derive(Clone)]
pub(crate) struct FallingKey {
    key: char,
    x: f64,
    y: f64,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/falling_keys_game.ui")]
    pub struct FallingKeysGame {
        #[template_child]
        pub game_area: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub score_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub difficulty_label: TemplateChild<gtk::Label>,

        pub drawing_area: RefCell<Option<DrawingArea>>,
        pub keyboard_widget: RefCell<Option<crate::keyboard_widget::KeyboardWidget>>,
        pub(crate) falling_keys: Rc<RefCell<Vec<FallingKey>>>,
        pub score: RefCell<u32>,
        pub difficulty: RefCell<u32>,
        pub speed: RefCell<f64>,
        pub game_over: RefCell<bool>,
        pub game_loop_running: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FallingKeysGame {
        const NAME: &'static str = "FallingKeysGame";
        type Type = super::FallingKeysGame;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FallingKeysGame {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_game();
        }
    }
    impl WidgetImpl for FallingKeysGame {}
    impl BoxImpl for FallingKeysGame {}
}

glib::wrapper! {
    pub struct FallingKeysGame(ObjectSubclass<imp::FallingKeysGame>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl FallingKeysGame {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_game(&self) {
        let imp = self.imp();

        // Create drawing area for falling keys
        let drawing_area = DrawingArea::new();
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);
        drawing_area.set_can_focus(true);
        drawing_area.set_focusable(true);

        let falling_keys = imp.falling_keys.clone();
        drawing_area.set_draw_func(move |_, cr, _width, _height| {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.paint().unwrap();

            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_font_size(24.0);

            for key in falling_keys.borrow().iter() {
                cr.move_to(key.x, key.y);
                cr.show_text(&key.key.to_string()).unwrap();
            }
        });

        imp.game_area.set_child(Some(&drawing_area));

        // Create keyboard widget
        let keyboard = crate::keyboard_widget::KeyboardWidget::new();
        imp.game_area.add_overlay(keyboard.widget());
        keyboard.widget().set_valign(gtk::Align::End);
        keyboard.widget().set_margin_bottom(20);
        imp.keyboard_widget.replace(Some(keyboard));

        // Add falling keys overlay on top of keyboard
        let keys_overlay = DrawingArea::new();
        keys_overlay.set_vexpand(true);
        keys_overlay.set_hexpand(true);
        keys_overlay.set_can_focus(true);
        keys_overlay.set_focusable(true);

        let falling_keys_clone = imp.falling_keys.clone();
        keys_overlay.set_draw_func(move |_, cr, _width, _height| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_font_size(24.0);

            for key in falling_keys_clone.borrow().iter() {
                cr.move_to(key.x, key.y);
                cr.show_text(&key.key.to_string()).unwrap();
            }
        });

        imp.game_area.add_overlay(&keys_overlay);
        imp.drawing_area.replace(Some(keys_overlay.clone()));

        // Setup keyboard input on the keys overlay
        let key_controller = gtk::EventControllerKey::new();
        let obj = self.downgrade();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if let Some(obj) = obj.upgrade() {
                if let Some(c) = key.to_unicode() {
                    obj.handle_key_press(c.to_ascii_lowercase());
                }
            }
            glib::Propagation::Stop
        });
        keys_overlay.add_controller(key_controller);

        // Grab focus when shown
        keys_overlay.grab_focus();

        // Start game loop
        self.start_game_loop();
    }

    fn start_game_loop(&self) {
        let imp = self.imp();

        // Don't start if already running
        if *imp.game_loop_running.borrow() {
            return;
        }
        *imp.game_loop_running.borrow_mut() = true;

        let obj = self.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            if let Some(obj) = obj.upgrade() {
                if *obj.imp().game_over.borrow() {
                    *obj.imp().game_loop_running.borrow_mut() = false;
                    return glib::ControlFlow::Break;
                }
                obj.update_game();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        // Spawn new keys
        let obj = self.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
            if let Some(obj) = obj.upgrade() {
                if *obj.imp().game_over.borrow() {
                    return glib::ControlFlow::Break;
                }
                obj.spawn_key();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    fn spawn_key(&self) {
        let imp = self.imp();
        let mut rng = rand::thread_rng();

        if let Some(drawing_area) = imp.drawing_area.borrow().as_ref() {
            let width = drawing_area.width() as f64;
            if width > 100.0 {
                let key = KEYS[rng.gen_range(0..KEYS.len())];

                imp.falling_keys.borrow_mut().push(FallingKey {
                    key,
                    x: rng.gen_range(50.0..width - 50.0),
                    y: 0.0,
                });
            }
        }
    }

    fn update_game(&self) {
        let imp = self.imp();
        let speed = *imp.speed.borrow();

        if let Some(drawing_area) = imp.drawing_area.borrow().as_ref() {
            let height = drawing_area.height() as f64;
            let mut keys = imp.falling_keys.borrow_mut();

            // Update positions
            for key in keys.iter_mut() {
                key.y += speed;
            }

            // Check for game over - key reached bottom of view
            if keys.iter().any(|k| k.y > height) {
                *imp.game_over.borrow_mut() = true;
                self.show_game_over();
            }

            drawing_area.queue_draw();
        }
    }

    fn handle_key_press(&self, key: char) {
        let imp = self.imp();

        // Highlight key on keyboard
        if let Some(keyboard) = imp.keyboard_widget.borrow().as_ref() {
            keyboard.set_current_key(Some(key));

            let keyboard_clone = keyboard.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                keyboard_clone.set_current_key(None);
            });
        }

        let mut keys = imp.falling_keys.borrow_mut();

        if let Some(pos) = keys.iter().position(|k| k.key == key) {
            keys.remove(pos);

            let mut score = imp.score.borrow_mut();
            *score += 1;
            imp.score_label
                .set_text(&i18n_fmt! { i18n_fmt("Score: {score}") });

            // Increase difficulty every 10 points
            if (*score).is_multiple_of(10) {
                let mut difficulty = imp.difficulty.borrow_mut();
                *difficulty += 1;
                imp.difficulty_label
                    .set_text(&i18n_fmt! { i18n_fmt("Level: {difficulty}") });

                let mut speed = imp.speed.borrow_mut();
                *speed += 0.5;
            }

            if let Some(drawing_area) = imp.drawing_area.borrow().as_ref() {
                drawing_area.queue_draw();
            }
        } else {
            // Wrong key pressed - decrease score
            let mut score = imp.score.borrow_mut();
            if *score > 0 {
                *score -= 1;
                imp.score_label
                    .set_text(&i18n_fmt! { i18n_fmt("Score: {score}") });
            }
        }
    }

    fn show_game_over(&self) {
        let imp = self.imp();
        *imp.game_over.borrow_mut() = true;

        // Hide game area and show results
        if let Some(child) = imp.game_area.child() {
            child.set_visible(false);
        }
        if let Some(drawing_area) = imp.drawing_area.borrow().as_ref() {
            drawing_area.set_visible(false);
        }
        if let Some(keyboard) = imp.keyboard_widget.borrow().as_ref() {
            keyboard.widget().set_visible(false);
        }

        let score = *imp.score.borrow();
        let level = *imp.difficulty.borrow();

        // Create results view
        let results_box = gtk::Box::new(gtk::Orientation::Vertical, 36);
        results_box.set_halign(gtk::Align::Center);
        results_box.set_valign(gtk::Align::Center);
        results_box.set_vexpand(true);

        // Score and level display
        let stats_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        // Score
        let score_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        score_box.set_width_request(200);
        let score_label = gtk::Label::new(Some(&score.to_string()));
        score_label.add_css_class("title-1");
        let score_desc = gtk::Label::new(Some("Score"));
        score_desc.add_css_class("dim-label");
        score_box.append(&score_label);
        score_box.append(&score_desc);

        let separator = gtk::Separator::new(gtk::Orientation::Vertical);

        // Level
        let level_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        level_box.set_width_request(200);
        let level_label = gtk::Label::new(Some(&level.to_string()));
        level_label.add_css_class("title-1");
        let level_desc = gtk::Label::new(Some("Level Reached"));
        level_desc.add_css_class("dim-label");
        level_box.append(&level_label);
        level_box.append(&level_desc);

        stats_box.append(&score_box);
        stats_box.append(&separator);
        stats_box.append(&level_box);

        // Restart button
        let restart_button = gtk::Button::with_label("Play Again");
        restart_button.add_css_class("pill");
        restart_button.add_css_class("suggested-action");

        let obj = self.downgrade();
        restart_button.connect_clicked(move |_| {
            if let Some(obj) = obj.upgrade() {
                obj.restart_game();
            }
        });

        results_box.append(&stats_box);
        results_box.append(&restart_button);

        imp.game_area.add_overlay(&results_box);
    }

    fn restart_game(&self) {
        let imp = self.imp();

        // Remove results overlay
        let mut child = imp.game_area.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling();
            if widget.type_() == gtk::Box::static_type() {
                imp.game_area.remove_overlay(&widget);
            }
            child = next;
        }

        // Show game elements
        if let Some(child) = imp.game_area.child() {
            child.set_visible(true);
        }
        if let Some(drawing_area) = imp.drawing_area.borrow().as_ref() {
            drawing_area.set_visible(true);
            drawing_area.grab_focus();
        }
        if let Some(keyboard) = imp.keyboard_widget.borrow().as_ref() {
            keyboard.widget().set_visible(true);
        }

        self.reset();
    }

    pub fn reset(&self) {
        let imp = self.imp();
        imp.falling_keys.borrow_mut().clear();
        *imp.score.borrow_mut() = 0;
        *imp.difficulty.borrow_mut() = 1;
        *imp.speed.borrow_mut() = 2.0;
        *imp.game_over.borrow_mut() = false;

        imp.score_label.set_text(&gettext("Score: 0"));
        imp.difficulty_label.set_text(&gettext("Level: 1"));

        if let Some(drawing_area) = imp.drawing_area.borrow().as_ref() {
            drawing_area.grab_focus();
            drawing_area.queue_draw();
        }

        self.start_game_loop();
    }
}
