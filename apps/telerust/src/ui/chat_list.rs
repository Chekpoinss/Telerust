use adw::prelude::*;
use telerust_tdlib_bridge::ChatSummary;

#[derive(Clone)]
pub struct ChatList {
    pub widget: gtk::ListBox,
    pub search: gtk::SearchEntry,
}

impl ChatList {
    pub fn new() -> Self {
        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some("Search chats"));
        search.set_margin_start(10);
        search.set_margin_end(10);
        search.set_margin_bottom(8);
        let widget = gtk::ListBox::new();
        widget.set_selection_mode(gtk::SelectionMode::Single);
        widget.add_css_class("chat-list");
        let filter_text = search.clone();
        widget.set_filter_func(move |row| {
            let query = filter_text.text().to_string().to_lowercase();
            query.is_empty()
                || row
                    .child()
                    .is_some_and(|child| child.widget_name().to_lowercase().contains(&query))
        });
        let list = widget.clone();
        search.connect_search_changed(move |_| list.invalidate_filter());
        Self { widget, search }
    }

    pub fn render(&self, chats: Vec<ChatSummary>, selected_id: Option<i64>) {
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }
        for chat in chats {
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(&chat.id.to_string());
            row.set_tooltip_text(Some(&chat.title));
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            content.set_margin_start(12);
            content.set_margin_end(12);
            content.set_margin_top(9);
            content.set_margin_bottom(9);
            let avatar = gtk::Label::new(Some(
                &chat
                    .title
                    .chars()
                    .next()
                    .unwrap_or('•')
                    .to_uppercase()
                    .to_string(),
            ));
            avatar.add_css_class("avatar-placeholder");
            content.append(&avatar);
            let text_column = gtk::Box::new(gtk::Orientation::Vertical, 4);
            text_column.set_hexpand(true);
            let title = gtk::Label::new(Some(&chat.title));
            title.set_xalign(0.0);
            title.set_ellipsize(gtk::pango::EllipsizeMode::End);
            title.add_css_class("heading");
            let preview = gtk::Label::new(Some(&chat.preview));
            preview.set_xalign(0.0);
            preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
            preview.add_css_class("dim-label");
            text_column.append(&title);
            text_column.append(&preview);
            content.append(&text_column);
            content.set_widget_name(&format!("{} {}", chat.title, chat.preview));
            row.set_child(Some(&content));
            self.widget.append(&row);
            if selected_id == Some(chat.id) {
                self.widget.select_row(Some(&row));
            }
        }
    }
}
