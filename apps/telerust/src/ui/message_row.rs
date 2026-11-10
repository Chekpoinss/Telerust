use adw::prelude::*;
use std::{cell::Cell, rc::Rc};
use telerust_client_core::history::{DeliveryState, Message};

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

pub fn factory() -> (gtk::SignalListItemFactory, FactoryMetrics) {
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
                let text = gtk::Label::new(Some(&message.text));
                text.set_wrap(true);
                text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                text.set_max_width_chars(72);
                text.set_xalign(0.0);
                text.set_selectable(true);
                bubble.append(&text);
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

fn format_time(timestamp: i64) -> String {
    gtk::glib::DateTime::from_unix_local(timestamp)
        .and_then(|time| time.format("%H:%M"))
        .map(|formatted| formatted.to_string())
        .unwrap_or_default()
}
