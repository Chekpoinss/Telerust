use adw::prelude::*;

#[derive(Clone)]
pub struct ChatHeader {
    pub widget: gtk::Box,
    pub name: gtk::Label,
    pub avatar: gtk::Label,
}

impl ChatHeader {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        widget.add_css_class("chat-header");
        let avatar = gtk::Label::new(Some("•"));
        avatar.add_css_class("avatar-placeholder");
        widget.append(&avatar);
        let identity = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let name = gtk::Label::new(Some("Select a chat"));
        name.set_xalign(0.0);
        name.add_css_class("heading");
        let presence = gtk::Label::new(Some("Telegram"));
        presence.set_xalign(0.0);
        presence.add_css_class("dim-label");
        identity.append(&name);
        identity.append(&presence);
        widget.append(&identity);
        widget.set_hexpand(true);
        let find = gtk::Button::from_icon_name("system-search-symbolic");
        find.add_css_class("flat");
        widget.append(&find);
        let info = gtk::Button::from_icon_name("view-more-symbolic");
        info.add_css_class("flat");
        widget.append(&info);
        Self {
            widget,
            name,
            avatar,
        }
    }

    pub fn set_chat(&self, title: &str) {
        self.name.set_text(title);
        self.avatar.set_text(
            &title
                .chars()
                .next()
                .unwrap_or('•')
                .to_uppercase()
                .to_string(),
        );
    }
}
