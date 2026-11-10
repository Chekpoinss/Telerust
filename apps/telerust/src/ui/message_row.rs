use adw::prelude::*;
use std::fmt::Write as _;
use std::{cell::Cell, rc::Rc};
use telerust_client_core::history::{DeliveryState, MediaFile, Message, MessageKind};
use telerust_tdlib_bridge::BackendCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineItem {
    Date(String),
    Message {
        message: Message,
        grouped: bool,
        show_metadata: bool,
    },
}

#[derive(Clone, Default)]
pub struct FactoryMetrics {
    current: Rc<Cell<usize>>,
    peak: Rc<Cell<usize>>,
    created: Rc<Cell<usize>>,
}

impl FactoryMetrics {
    pub fn realized(&self) -> usize {
        self.current.get()
    }
    pub fn peak_realized(&self) -> usize {
        self.peak.get()
    }
    pub fn created(&self) -> usize {
        self.created.get()
    }
}

pub fn factory(
    command_tx: async_channel::Sender<BackendCommand>,
) -> (gtk::SignalListItemFactory, FactoryMetrics) {
    let factory = gtk::SignalListItemFactory::new();
    let metrics = FactoryMetrics::default();
    let created = metrics.created.clone();
    factory.connect_setup(move |_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().expect("ListItem");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        row.set_margin_start(12);
        row.set_margin_end(12);
        row.set_margin_top(3);
        row.set_margin_bottom(3);
        item.set_child(Some(&row));
        created.set(created.get() + 1);
    });
    let current = metrics.current.clone();
    let peak = metrics.peak.clone();
    let download_tx = command_tx.clone();
    factory.connect_bind(move |_, item| {
        let count = current.get() + 1;
        current.set(count);
        peak.set(peak.get().max(count));
        let item = item.downcast_ref::<gtk::ListItem>().expect("ListItem");
        let Some(object) = item.item().and_downcast::<gtk::glib::BoxedAnyObject>() else {
            return;
        };
        let data = object.borrow::<TimelineItem>().clone();
        let row = item
            .child()
            .and_downcast::<gtk::Box>()
            .expect("message row");
        while let Some(child) = row.first_child() {
            row.remove(&child);
        }
        match data {
            TimelineItem::Date(label) => {
                let pill = gtk::Label::new(Some(&label));
                pill.add_css_class("date-separator");
                pill.set_halign(gtk::Align::Center);
                row.append(&pill);
                row.set_halign(gtk::Align::Center);
            }
            TimelineItem::Message {
                message,
                grouped,
                show_metadata,
            } => {
                let bubble = gtk::Box::new(gtk::Orientation::Vertical, 2);
                bubble.set_margin_start(8);
                bubble.set_margin_end(8);
                bubble.set_margin_top(6);
                bubble.set_margin_bottom(6);
                bubble.add_css_class(if message.outgoing {
                    "message-out"
                } else {
                    "message-in"
                });
                if grouped {
                    bubble.add_css_class(if message.outgoing {
                        "message-grouped-out"
                    } else {
                        "message-grouped-in"
                    });
                }
                bubble.set_halign(if message.outgoing {
                    gtk::Align::End
                } else {
                    gtk::Align::Start
                });
                bubble.set_hexpand(false);
                bubble.set_size_request(84, -1);
                match &message.kind {
                    MessageKind::Text if !message.text.trim().is_empty() => {
                        let text = gtk::Label::new(Some(&message.text));
                        text.set_markup(&text_markup(&message.text));
                        text.set_wrap(true);
                        text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                        text.set_max_width_chars(72);
                        text.set_xalign(0.0);
                        text.set_selectable(true);
                        bubble.append(&text);
                    }
                    MessageKind::Text => append_info(&bubble, "Empty text message"),
                    MessageKind::Photo(file) => {
                        append_media(&bubble, file, "Photo", &download_tx, true)
                    }
                    MessageKind::Document(file) => {
                        append_media(&bubble, file, "Document", &download_tx, false)
                    }
                    MessageKind::Video(file) => {
                        append_media(&bubble, file, "Video", &download_tx, false)
                    }
                    MessageKind::Sticker(file) => {
                        append_media(&bubble, file, "Sticker", &download_tx, true)
                    }
                    MessageKind::Voice(file) => {
                        append_media(&bubble, file, "Voice message", &download_tx, false)
                    }
                    MessageKind::Audio(file) => {
                        append_media(&bubble, file, "Audio", &download_tx, false)
                    }
                    MessageKind::Service(label) | MessageKind::Unsupported(label) => {
                        append_info(&bubble, label)
                    }
                }
                if show_metadata {
                    let metadata = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                    metadata.set_halign(gtk::Align::End);
                    let time = gtk::Label::new(Some(&format_time(message.date)));
                    time.add_css_class("message-time");
                    metadata.append(&time);
                    if message.outgoing {
                        let state = match message.delivery {
                            DeliveryState::Sending => "◷",
                            DeliveryState::Failed => "!",
                            DeliveryState::Sent => "✓✓",
                            DeliveryState::Received => "",
                        };
                        let status = gtk::Label::new(Some(state));
                        status.add_css_class("message-time");
                        metadata.append(&status);
                    }
                    bubble.append(&metadata);
                }
                row.append(&bubble);
                row.set_halign(gtk::Align::Fill);
            }
        }
    });
    let current = metrics.current.clone();
    factory.connect_unbind(move |_, _| current.set(current.get().saturating_sub(1)));
    (factory, metrics)
}

fn append_info(bubble: &gtk::Box, text: &str) {
    let label = gtk::Label::new(Some(text));
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_max_width_chars(64);
    label.add_css_class("dim-label");
    bubble.append(&label);
}

fn text_markup(text: &str) -> String {
    let mut markup = String::with_capacity(text.len());
    for chunk in text.split_inclusive(char::is_whitespace) {
        let word = chunk.trim_end_matches(char::is_whitespace);
        let trailing = &chunk[word.len()..];
        let link_end = word
            .char_indices()
            .rev()
            .find(|(_, ch)| !matches!(ch, '.' | ',' | ';' | '!' | '?' | ')' | ']'))
            .map_or(0, |(index, ch)| index + ch.len_utf8());
        let candidate = &word[..link_end];
        if (candidate.starts_with("https://") || candidate.starts_with("http://"))
            && candidate
                .split_once("://")
                .is_some_and(|(_, host)| !host.is_empty() && !host.starts_with('/'))
        {
            let href = gtk::glib::markup_escape_text(candidate);
            let title = gtk::glib::markup_escape_text(candidate);
            let _ = write!(markup, "<a href=\"{href}\">{title}</a>");
            markup.push_str(gtk::glib::markup_escape_text(&word[link_end..]).as_str());
        } else {
            markup.push_str(gtk::glib::markup_escape_text(word).as_str());
        }
        markup.push_str(gtk::glib::markup_escape_text(trailing).as_str());
    }
    markup
}

fn append_media(
    bubble: &gtk::Box,
    file: &MediaFile,
    title: &str,
    tx: &async_channel::Sender<BackendCommand>,
    show_preview: bool,
) {
    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let icon = gtk::Image::from_icon_name(media_icon(file, title));
    icon.set_pixel_size(24);
    heading.append(&icon);
    let details = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let name = gtk::Label::new(Some(&file.name));
    name.set_xalign(0.0);
    name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    name.set_max_width_chars(48);
    name.add_css_class("heading");
    details.append(&name);
    let mut meta = if file.size > 0 {
        format!("{} · {}", title, format_size(file.size))
    } else {
        title.to_owned()
    };
    if file.duration > 0 {
        meta.push_str(&format!(" · {}s", file.duration));
    }
    if file.downloading {
        meta.push_str(&format!(" · {}", format_size(file.downloaded_size)));
        if file.size > 0 {
            meta.push_str(&format!(" / {}", format_size(file.size)));
        }
    }
    let subtitle = gtk::Label::new(Some(&meta));
    subtitle.set_xalign(0.0);
    subtitle.add_css_class("dim-label");
    details.append(&subtitle);
    heading.append(&details);
    bubble.append(&heading);

    if show_preview {
        if let Some(path) = file.local_path.as_deref().filter(|_| file.downloaded) {
            if std::path::Path::new(path).is_file() {
                let picture = gtk::Picture::for_filename(path);
                picture.set_content_fit(gtk::ContentFit::Contain);
                picture.set_can_shrink(true);
                picture.set_size_request(240, 120);
                picture.set_halign(gtk::Align::Start);
                bubble.append(&picture);
                let open = gtk::Button::with_label("Open image file");
                let uri = gtk::gio::File::for_path(path).uri();
                open.connect_clicked(move |_| {
                    let _ = gtk::gio::AppInfo::launch_default_for_uri(
                        &uri,
                        None::<&gtk::gio::AppLaunchContext>,
                    );
                });
                bubble.append(&open);
                return;
            }
        }
    }

    let action = if file.downloading {
        "Downloading…"
    } else if file.downloaded {
        "Open file"
    } else if show_preview {
        "Download preview"
    } else {
        "Download"
    };
    let button = gtk::Button::with_label(action);
    if file.downloading {
        button.set_sensitive(false);
    } else if file.downloaded {
        if let Some(path) = file.local_path.as_deref() {
            let uri = gtk::gio::File::for_path(path).uri();
            button.connect_clicked(move |_| {
                let _ = gtk::gio::AppInfo::launch_default_for_uri(
                    &uri,
                    None::<&gtk::gio::AppLaunchContext>,
                );
            });
        }
    } else {
        let tx = tx.clone();
        let file_id = file.id;
        button.connect_clicked(move |_| {
            let _ = tx.try_send(BackendCommand::DownloadFile { file_id });
        });
    }
    button.set_halign(gtk::Align::Start);
    bubble.append(&button);
}

fn media_icon(file: &MediaFile, title: &str) -> &'static str {
    if title == "Photo" || file.mime_type.starts_with("image/") {
        "image-x-generic-symbolic"
    } else if file.mime_type.starts_with("video/") {
        "video-x-generic-symbolic"
    } else if file.mime_type.starts_with("audio/") {
        "audio-x-generic-symbolic"
    } else {
        match file
            .name
            .rsplit_once('.')
            .map(|(_, extension)| extension.to_ascii_lowercase())
        {
            Some(extension) if extension == "pdf" => "application-pdf-symbolic",
            Some(extension)
                if matches!(extension.as_str(), "zip" | "tar" | "gz" | "7z" | "rar") =>
            {
                "package-x-generic-symbolic"
            }
            Some(extension) if matches!(extension.as_str(), "doc" | "docx" | "odt" | "rtf") => {
                "x-office-document-symbolic"
            }
            Some(extension) if matches!(extension.as_str(), "xls" | "xlsx" | "ods" | "csv") => {
                "x-office-spreadsheet-symbolic"
            }
            Some(extension) if matches!(extension.as_str(), "ppt" | "pptx" | "odp") => {
                "x-office-presentation-symbolic"
            }
            _ => "text-x-generic-symbolic",
        }
    }
}

fn format_size(size: i64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = size.max(0) as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn format_time(timestamp: i64) -> String {
    gtk::glib::DateTime::from_unix_local(timestamp)
        .and_then(|time| time.format("%H:%M"))
        .map(|formatted| formatted.to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::text_markup;

    #[test]
    fn text_links_are_http_only_and_markup_escaped() {
        assert_eq!(
            text_markup("see https://example.test/path, <hello>"),
            "see <a href=\"https://example.test/path\">https://example.test/path</a>, &lt;hello&gt;"
        );
        assert_eq!(text_markup("javascript:alert(1)"), "javascript:alert(1)");
    }
}
