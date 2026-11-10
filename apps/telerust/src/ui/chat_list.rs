use adw::prelude::*;
use std::{cell::RefCell, rc::Rc};
use telerust_tdlib_bridge::{BackendCommand, ChatSummary};

#[derive(Clone)]
pub struct ChatList {
    pub widget: gtk::ListView,
    pub search: gtk::SearchEntry,
    store: gtk::gio::ListStore,
    selection: gtk::SingleSelection,
}

impl ChatList {
    pub fn new(command_tx: async_channel::Sender<BackendCommand>) -> Self {
        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some("Search chats"));
        search.set_margin_start(10);
        search.set_margin_end(10);
        search.set_margin_bottom(8);

        let store = gtk::gio::ListStore::new::<gtk::glib::BoxedAnyObject>();
        let query = Rc::new(RefCell::new(String::new()));
        let filter_query = query.clone();
        let filter = gtk::CustomFilter::new(move |object| {
            let Some(item) = object.downcast_ref::<gtk::glib::BoxedAnyObject>() else {
                return false;
            };
            let chat = item.borrow::<ChatSummary>();
            let query = filter_query.borrow();
            query.is_empty()
                || chat.title.to_lowercase().contains(&*query)
                || chat.preview.to_lowercase().contains(&*query)
        });
        let filtered = gtk::FilterListModel::new(Some(store.clone()), Some(filter.clone()));
        let selection = gtk::SingleSelection::new(Some(filtered));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().expect("ListItem");
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            row.set_margin_start(12);
            row.set_margin_end(12);
            row.set_margin_top(8);
            row.set_margin_bottom(8);
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
            row.append(&avatar);
            let details = gtk::Box::new(gtk::Orientation::Vertical, 4);
            details.set_hexpand(true);
            let title = gtk::Label::new(None);
            title.set_xalign(0.0);
            title.set_ellipsize(gtk::pango::EllipsizeMode::End);
            title.add_css_class("heading");
            details.append(&title);
            let preview = gtk::Label::new(None);
            preview.set_xalign(0.0);
            preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
            preview.add_css_class("dim-label");
            details.append(&preview);
            row.append(&details);
            item.set_child(Some(&row));
        });

        let tx = command_tx;
        factory.connect_bind(move |_, item| {
            let Some(item) = item.downcast_ref::<gtk::ListItem>() else {
                return;
            };
            let Some(data) = item.item().and_downcast::<gtk::glib::BoxedAnyObject>() else {
                return;
            };
            let chat = data.borrow::<ChatSummary>().clone();
            let Some(row) = item.child().and_downcast::<gtk::Box>() else {
                return;
            };
            let Some(avatar) = row.first_child().and_downcast::<gtk::Stack>() else {
                return;
            };
            let Some(details) = avatar.next_sibling().and_downcast::<gtk::Box>() else {
                return;
            };
            let Some(title) = details.first_child().and_downcast::<gtk::Label>() else {
                return;
            };
            let Some(preview) = title.next_sibling().and_downcast::<gtk::Label>() else {
                return;
            };
            title.set_text(&chat.title);
            preview.set_text(&chat.preview);
            let initials = chat
                .title
                .chars()
                .next()
                .unwrap_or('•')
                .to_uppercase()
                .to_string();
            if let Some(fallback) = avatar
                .child_by_name("fallback")
                .and_downcast::<gtk::Label>()
            {
                fallback.set_text(&initials);
            }
            if let Some(path) = chat
                .photo_path
                .as_deref()
                .filter(|path| std::path::Path::new(path).is_file())
            {
                if let Some(image) = avatar.child_by_name("photo").and_downcast::<gtk::Picture>() {
                    image.set_filename(Some(path));
                }
                avatar.set_visible_child_name("photo");
            } else {
                avatar.set_visible_child_name("fallback");
                if let Some(image) = avatar.child_by_name("photo").and_downcast::<gtk::Picture>() {
                    image.set_filename(None::<&std::path::Path>);
                }
                if let Some(file_id) = chat.photo_file_id {
                    let _ = tx.try_send(BackendCommand::DownloadFile { file_id });
                }
            }
            row.set_tooltip_text(Some(&chat.title));
        });

        let widget = gtk::ListView::new(Some(selection.clone()), Some(factory));
        widget.add_css_class("chat-list");
        let query = query.clone();
        let filter = filter.clone();
        search.connect_search_changed(move |entry| {
            *query.borrow_mut() = entry.text().trim().to_lowercase();
            filter.changed(gtk::FilterChange::Different);
        });
        Self {
            widget,
            search,
            store,
            selection,
        }
    }

    pub fn render(&self, chats: Vec<ChatSummary>, selected_id: Option<i64>) {
        self.store.remove_all();
        for chat in chats {
            self.store.append(&gtk::glib::BoxedAnyObject::new(chat));
        }
        let selected = selected_id.and_then(|selected_id| {
            (0..self.selection.n_items()).find(|index| {
                self.chat_at(*index)
                    .is_some_and(|chat| chat.id == selected_id)
            })
        });
        if let Some(index) = selected {
            self.selection.set_selected(index);
        } else {
            self.selection.unselect_all();
        }
    }

    pub fn chat_at(&self, filtered_position: u32) -> Option<ChatSummary> {
        self.selection
            .item(filtered_position)
            .and_downcast::<gtk::glib::BoxedAnyObject>()
            .map(|item| item.borrow::<ChatSummary>().clone())
    }
}
