use adw::prelude::*;

#[derive(Clone)]
pub struct Composer {
    pub widget: gtk::Box,
    pub input: gtk::Entry,
    pub send: gtk::Button,
}

impl Composer {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_margin_top(8);
        widget.set_margin_bottom(12);
        let input = gtk::Entry::new();
        input.set_hexpand(true);
        input.set_placeholder_text(Some("Message"));
        let send = gtk::Button::with_label("Send");
        send.add_css_class("suggested-action");
        widget.append(&input);
        widget.append(&send);
        Self {
            widget,
            input,
            send,
        }
    }
}
