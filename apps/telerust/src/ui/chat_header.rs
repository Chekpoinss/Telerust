use adw::prelude::*;
use telerust_tdlib_bridge::ChatSummary;

#[derive(Clone)]
pub struct ChatHeader {
    pub widget: gtk::Box,
    pub name: gtk::Label,
    pub avatar: gtk::Stack,
}

impl ChatHeader {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        widget.add_css_class("chat-header");
        let avatar = gtk::Stack::new();
        avatar.set_size_request(44, 44);
        avatar.add_css_class("chat-avatar");
        let fallback = gtk::Label::new(Some("•"));
        fallback.add_css_class("avatar-placeholder");
        avatar.add_named(&fallback, Some("fallback"));
        let image = gtk::Picture::new();
        image.set_content_fit(gtk::ContentFit::Cover);
        image.set_can_shrink(true);
        image.set_size_request(44, 44);
        avatar.add_named(&image, Some("photo"));
        widget.append(&avatar);
        let identity = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let name = gtk::Label::new(Some("Select a chat"));
        name.set_xalign(0.0);
        name.add_css_class("heading");
        let presence = gtk::Label::new(Some("Conversation"));
        presence.set_xalign(0.0);
        presence.add_css_class("dim-label");
        identity.append(&name);
        identity.append(&presence);
        widget.append(&identity);
        widget.set_hexpand(true);
        Self {
            widget,
            name,
            avatar,
        }
    }

    pub fn set_chat(&self, chat: &ChatSummary) {
        self.name.set_text(&chat.title);
        if let Some(fallback) = self
            .avatar
            .child_by_name("fallback")
            .and_downcast::<gtk::Label>()
        {
            fallback.set_text(
                &chat
                    .title
                    .chars()
                    .next()
                    .unwrap_or('•')
                    .to_uppercase()
                    .to_string(),
            );
        }
        if let Some(path) = chat
            .photo_path
            .as_deref()
            .filter(|path| std::path::Path::new(path).is_file())
        {
            if let Some(image) = self
                .avatar
                .child_by_name("photo")
                .and_downcast::<gtk::Picture>()
            {
                image.set_filename(Some(path));
            }
            self.avatar.set_visible_child_name("photo");
        } else {
            self.avatar.set_visible_child_name("fallback");
            if let Some(image) = self
                .avatar
                .child_by_name("photo")
                .and_downcast::<gtk::Picture>()
            {
                image.set_filename(None::<&std::path::Path>);
            }
        }
    }
}
