use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/target_text_view.ui")]
    pub struct TargetTextView {
        pub cursor_position: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TargetTextView {
        const NAME: &'static str = "MecalinTargetTextView";
        type Type = super::TargetTextView;
        type ParentType = gtk::TextView;

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
            self.obj().set_can_target(false);
            self.obj().set_cursor_visible(false);
            self.obj().set_monospace(true);
        }
    }

    impl WidgetImpl for TargetTextView {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            self.draw_cursor(snapshot);
        }
    }

    impl TextViewImpl for TargetTextView {}
}

impl imp::TargetTextView {
    fn draw_cursor(&self, snapshot: &gtk::Snapshot) {
        let cursor_pos = self.cursor_position.get();
        let buffer = self.obj().buffer();

        let mut iter = buffer.start_iter();
        iter.forward_chars(cursor_pos);
        let rect = self.obj().iter_location(&iter);

        let (x, y) =
            self.obj()
                .buffer_to_window_coords(gtk::TextWindowType::Widget, rect.x(), rect.y());

        #[allow(deprecated)]
        let style_ctx = self.obj().style_context();
        #[allow(deprecated)]
        let color = style_ctx.color();

        let cursor_rect = gtk::graphene::Rect::new(x as f32, y as f32, 2.0, rect.height() as f32);
        snapshot.append_color(&color, &cursor_rect);
    }
}

glib::wrapper! {
    pub struct TargetTextView(ObjectSubclass<imp::TargetTextView>)
        @extends gtk::TextView, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

impl TargetTextView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_text(&self, text: &str) {
        let buffer = self.buffer();
        buffer.set_text(text);
        buffer.place_cursor(&buffer.start_iter());
    }

    pub fn set_cursor_position(&self, position: i32) {
        let imp = self.imp();
        imp.cursor_position.set(position);
        self.queue_draw();
    }
}

impl Default for TargetTextView {
    fn default() -> Self {
        Self::new()
    }
}
