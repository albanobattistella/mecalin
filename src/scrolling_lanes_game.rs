use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, DrawingArea};
use i18n_format::i18n_fmt;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

const WORDS: &[&str] = &[
    "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "hello", "world", "rust",
    "code", "type", "fast", "game", "play", "win", "lose", "start", "end",
];

#[derive(Clone)]
pub(crate) struct ScrollingText {
    text: String,
    x: f64,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/scrolling_lanes_game.ui")]
    pub struct ScrollingLanesGame {
        #[template_child]
        pub game_area: TemplateChild<gtk::Box>,
        #[template_child]
        pub score_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub level_label: TemplateChild<gtk::Label>,

        pub lanes: Rc<RefCell<Vec<DrawingArea>>>,
        pub(crate) lane_texts: Rc<RefCell<Vec<Vec<ScrollingText>>>>,
        pub current_lane: Rc<RefCell<usize>>,
        pub score: RefCell<u32>,
        pub difficulty: RefCell<u32>,
        pub speed: RefCell<f64>,
        pub game_over: RefCell<bool>,
        pub game_loop_running: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScrollingLanesGame {
        const NAME: &'static str = "ScrollingLanesGame";
        type Type = super::ScrollingLanesGame;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScrollingLanesGame {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_game();
        }
    }
    impl WidgetImpl for ScrollingLanesGame {}
    impl BoxImpl for ScrollingLanesGame {}
}

glib::wrapper! {
    pub struct ScrollingLanesGame(ObjectSubclass<imp::ScrollingLanesGame>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ScrollingLanesGame {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_game(&self) {
        let imp = self.imp();

        // Create 4 lanes
        let lanes_container = gtk::Box::new(gtk::Orientation::Vertical, 2);
        lanes_container.set_vexpand(true);
        lanes_container.set_hexpand(true);

        let mut lanes = Vec::new();
        let lane_texts = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        for i in 0..4 {
            let lane = DrawingArea::new();
            lane.set_vexpand(true);
            lane.set_hexpand(true);
            lane.set_height_request(100);

            let lane_index = i;
            let current_lane = imp.current_lane.clone();
            let texts = imp.lane_texts.clone();

            lane.set_draw_func(move |_, cr, width, _height| {
                let current = *current_lane.borrow();

                // Background
                if current == lane_index {
                    cr.set_source_rgb(0.2, 0.3, 0.4);
                } else {
                    cr.set_source_rgb(0.1, 0.1, 0.1);
                }
                cr.paint().unwrap();

                // Draw texts
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_font_size(20.0);

                if let Ok(all_texts) = texts.try_borrow() {
                    for text in &all_texts[lane_index] {
                        if text.x < width as f64 && text.x > -200.0 {
                            cr.move_to(text.x, 50.0);
                            cr.show_text(&text.text).unwrap();
                        }
                    }
                }
            });

            lanes_container.append(&lane);
            lanes.push(lane);
        }

        imp.game_area.append(&lanes_container);
        imp.lanes.replace(lanes);
        imp.lane_texts.replace(lane_texts);

        // Setup keyboard input
        let key_controller = gtk::EventControllerKey::new();
        let obj = self.downgrade();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if let Some(obj) = obj.upgrade() {
                obj.handle_key_press(key);
            }
            glib::Propagation::Stop
        });
        self.add_controller(key_controller);

        self.set_can_focus(true);
        self.set_focusable(true);

        // Grab focus after widget is realized
        let obj = self.downgrade();
        self.connect_realize(move |_| {
            if let Some(obj) = obj.upgrade() {
                obj.grab_focus();
            }
        });

        // Start game loop
        self.start_game_loop();
    }

    fn start_game_loop(&self) {
        let imp = self.imp();

        if *imp.game_loop_running.borrow() {
            return;
        }
        *imp.game_loop_running.borrow_mut() = true;

        // Update game
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

        // Spawn new texts
        let obj = self.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
            if let Some(obj) = obj.upgrade() {
                if *obj.imp().game_over.borrow() {
                    return glib::ControlFlow::Break;
                }
                obj.spawn_text();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    fn spawn_text(&self) {
        let imp = self.imp();
        let mut rng = rand::thread_rng();

        let lane_index = rng.gen_range(0..4);
        let word = WORDS[rng.gen_range(0..WORDS.len())];

        if let Some(lane) = imp.lanes.borrow().get(lane_index) {
            let width = lane.width() as f64;
            imp.lane_texts.borrow_mut()[lane_index].push(ScrollingText {
                text: word.to_string(),
                x: width,
            });
        }
    }

    fn update_game(&self) {
        let imp = self.imp();
        let speed = *imp.speed.borrow();

        let mut texts = imp.lane_texts.borrow_mut();
        let lanes = imp.lanes.borrow();

        for (lane_index, lane_texts) in texts.iter_mut().enumerate() {
            for text in lane_texts.iter_mut() {
                text.x -= speed;
            }

            // Check for game over
            if lane_texts.iter().any(|t| t.x < -200.0) {
                *imp.game_over.borrow_mut() = true;
                drop(texts);
                drop(lanes);
                self.show_game_over();
                return;
            }

            // Queue redraw
            if let Some(lane) = lanes.get(lane_index) {
                lane.queue_draw();
            }
        }
    }

    fn handle_key_press(&self, key: gtk::gdk::Key) {
        let imp = self.imp();
        let key_name = key.name();

        if key_name.as_deref() == Some("Up") {
            {
                let mut current = imp.current_lane.borrow_mut();
                if *current > 0 {
                    *current -= 1;
                }
            }
            // Redraw all lanes
            for lane in imp.lanes.borrow().iter() {
                lane.queue_draw();
            }
        } else if key_name.as_deref() == Some("Down") {
            {
                let mut current = imp.current_lane.borrow_mut();
                if *current < 3 {
                    *current += 1;
                }
            }
            // Redraw all lanes
            for lane in imp.lanes.borrow().iter() {
                lane.queue_draw();
            }
        } else if let Some(c) = key.to_unicode() {
            // Type to clear text in current lane
            self.handle_typing(c);
            // Redraw all lanes
            for lane in imp.lanes.borrow().iter() {
                lane.queue_draw();
            }
        }
    }

    fn handle_typing(&self, c: char) {
        let imp = self.imp();
        let current_lane = *imp.current_lane.borrow();

        let (found, score_changed) = {
            let mut texts = imp.lane_texts.borrow_mut();

            if let Some(lane_texts) = texts.get_mut(current_lane) {
                // Find leftmost text that starts with this character
                if let Some(pos) = lane_texts.iter().position(|t| t.text.starts_with(c)) {
                    // Remove first character from the text
                    let text = &mut lane_texts[pos].text;
                    text.remove(0);

                    // If text is now empty, remove it completely
                    if text.is_empty() {
                        lane_texts.remove(pos);
                    }

                    (true, true)
                } else {
                    (false, true)
                }
            } else {
                (false, false)
            }
        };

        if score_changed {
            let mut score = imp.score.borrow_mut();
            if found {
                *score += 1;
                let score_text = i18n_fmt! { i18n_fmt("Score: {}", *score) };
                imp.score_label.set_text(&score_text);

                if (*score).is_multiple_of(10) {
                    let mut difficulty = imp.difficulty.borrow_mut();
                    *difficulty += 1;
                    let level_text = i18n_fmt! { i18n_fmt("Level: {}", *difficulty) };
                    imp.level_label.set_text(&level_text);

                    let mut speed = imp.speed.borrow_mut();
                    *speed += 0.5;
                }
            } else if *score > 0 {
                *score -= 1;
                let score_text = i18n_fmt! { i18n_fmt("Score: {}", *score) };
                imp.score_label.set_text(&score_text);
            }
        }
    }

    fn show_game_over(&self) {
        let imp = self.imp();
        *imp.game_over.borrow_mut() = true;

        // Hide lanes
        if let Some(child) = imp.game_area.first_child() {
            child.set_visible(false);
        }

        let score = *imp.score.borrow();
        let level = *imp.difficulty.borrow();

        // Create results view
        let results_box = gtk::Box::new(gtk::Orientation::Vertical, 36);
        results_box.set_halign(gtk::Align::Center);
        results_box.set_valign(gtk::Align::Center);
        results_box.set_vexpand(true);

        let stats_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        let score_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        score_box.set_width_request(200);
        let score_label = gtk::Label::new(Some(&score.to_string()));
        score_label.add_css_class("title-1");
        let score_desc = gtk::Label::new(Some("Score"));
        score_desc.add_css_class("dim-label");
        score_box.append(&score_label);
        score_box.append(&score_desc);

        let separator = gtk::Separator::new(gtk::Orientation::Vertical);

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

        imp.game_area.append(&results_box);
    }

    fn restart_game(&self) {
        let imp = self.imp();

        // Remove results
        if let Some(child) = imp.game_area.last_child() {
            if child.type_() == gtk::Box::static_type() {
                imp.game_area.remove(&child);
            }
        }

        // Show lanes
        if let Some(child) = imp.game_area.first_child() {
            child.set_visible(true);
        }

        self.grab_focus();
        self.reset();
    }

    pub fn reset(&self) {
        let imp = self.imp();
        imp.lane_texts
            .borrow_mut()
            .iter_mut()
            .for_each(|v| v.clear());
        *imp.current_lane.borrow_mut() = 0;
        *imp.score.borrow_mut() = 0;
        *imp.difficulty.borrow_mut() = 1;
        *imp.speed.borrow_mut() = 2.0;
        *imp.game_over.borrow_mut() = false;

        imp.score_label.set_text(&gettext("Score: 0"));
        imp.level_label.set_text(&gettext("Level: 1"));

        for lane in imp.lanes.borrow().iter() {
            lane.queue_draw();
        }

        // Ensure focus
        glib::idle_add_local_once({
            let obj = self.clone();
            move || {
                obj.grab_focus();
            }
        });

        self.start_game_loop();
    }
}
