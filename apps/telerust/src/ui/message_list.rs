use super::message_row::{self, TimelineItem};
use adw::prelude::*;
use telerust_client_core::history::{HistoryWindow, Message};

#[derive(Clone)]
pub struct MessageList {
    pub scroller: gtk::ScrolledWindow,
    pub store: gtk::gio::ListStore,
    pub metrics: message_row::FactoryMetrics,
}

impl MessageList {
    pub fn new() -> Self {
        let store = gtk::gio::ListStore::new::<gtk::glib::BoxedAnyObject>();
        let selection = gtk::NoSelection::new(Some(store.clone()));
        let (factory, metrics) = message_row::factory();
        let view = gtk::ListView::new(Some(selection), Some(factory));
        view.set_vexpand(true);
        let scroller = gtk::ScrolledWindow::builder()
            .child(&view)
            .vexpand(true)
            .build();
        Self {
            scroller,
            store,
            metrics,
        }
    }

    pub fn render(&self, history: &HistoryWindow, preserve_scroll: bool) {
        let old_upper = self.scroller.vadjustment().upper();
        let old_value = self.scroller.vadjustment().value();
        let old_page_size = self.scroller.vadjustment().page_size();
        let was_at_bottom = old_upper - old_page_size - old_value < 48.0;
        let current: Vec<TimelineItem> = (0..self.store.n_items())
            .filter_map(|index| self.store.item(index))
            .filter_map(|object| object.downcast::<gtk::glib::BoxedAnyObject>().ok())
            .map(|object| object.borrow::<TimelineItem>().clone())
            .collect();
        let desired = timeline(history.messages());
        let old_first_id = current.iter().find_map(|item| match item {
            TimelineItem::Message { message, .. } => Some(message.id),
            TimelineItem::Date(_) => None,
        });
        let anchor_index = old_first_id.and_then(|id| {
            desired.iter().position(
                |item| matches!(item, TimelineItem::Message { message, .. } if message.id == id),
            )
        });
        let fixed_size_prepend = preserve_scroll
            && current.len() == desired.len()
            && anchor_index.is_some_and(|index| index > 0);
        let estimated_added_height = if fixed_size_prepend {
            estimate_anchor_offset(old_upper, current.len(), anchor_index.unwrap_or_default())
        } else {
            0.0
        };
        let mut prefix = 0;
        while prefix < current.len().min(desired.len()) && current[prefix] == desired[prefix] {
            prefix += 1;
        }
        let mut suffix = 0;
        while suffix
            < current
                .len()
                .saturating_sub(prefix)
                .min(desired.len().saturating_sub(prefix))
            && current[current.len() - suffix - 1] == desired[desired.len() - suffix - 1]
        {
            suffix += 1;
        }
        for _ in prefix..current.len().saturating_sub(suffix) {
            self.store.remove(prefix as u32);
        }
        for (offset, item) in desired[prefix..desired.len().saturating_sub(suffix)]
            .iter()
            .enumerate()
        {
            self.store.insert(
                (prefix + offset) as u32,
                &gtk::glib::BoxedAnyObject::new(item.clone()),
            );
        }
        let adjustment = self.scroller.vadjustment();
        gtk::glib::idle_add_local_once(move || {
            let delta = adjustment.upper() - old_upper;
            if old_upper == 0.0 || (!preserve_scroll && was_at_bottom) {
                adjustment.set_value((adjustment.upper() - adjustment.page_size()).max(0.0));
            } else if preserve_scroll {
                let added_height = if fixed_size_prepend {
                    estimated_added_height
                } else {
                    delta
                };
                adjustment.set_value((old_value + added_height).max(0.0));
            } else {
                adjustment.set_value(old_value.max(0.0));
            }
        });
    }
}

fn estimate_anchor_offset(old_upper: f64, old_item_count: usize, inserted_items: usize) -> f64 {
    if old_item_count == 0 {
        0.0
    } else {
        old_upper / old_item_count as f64 * inserted_items as f64
    }
}

fn timeline(messages: &[Message]) -> Vec<TimelineItem> {
    let mut items = Vec::with_capacity(messages.len() + 8);
    let mut last_day = None;
    for (index, message) in messages.iter().enumerate() {
        let day = local_day(message.date);
        if last_day.as_deref() != Some(day.as_str()) {
            let label = gtk::glib::DateTime::from_unix_local(message.date)
                .and_then(|date| date.format("%e %B %Y"))
                .map(|value| value.to_string())
                .unwrap_or_else(|_| format!("Day {day}"));
            items.push(TimelineItem::Date(label));
            last_day = Some(day.clone());
        }
        let previous = index.checked_sub(1).and_then(|i| messages.get(i));
        let next = messages.get(index + 1);
        let grouped = previous.is_some_and(|other| same_group(other, message));
        let show_metadata = next.is_none_or(|next| !same_group(next, message));
        items.push(TimelineItem::Message {
            message: message.clone(),
            grouped,
            show_metadata,
        });
    }
    items
}

fn same_group(left: &Message, right: &Message) -> bool {
    left.outgoing == right.outgoing
        && left.sender_id == right.sender_id
        && local_day(left.date) == local_day(right.date)
}

fn local_day(timestamp: i64) -> String {
    gtk::glib::DateTime::from_unix_local(timestamp)
        .and_then(|date| date.format("%Y-%m-%d"))
        .map(|value| value.to_string())
        .unwrap_or_else(|_| timestamp.div_euclid(86_400).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use telerust_client_core::history::DeliveryState;

    #[test]
    fn timeline_groups_adjacent_messages_and_marks_day_changes() {
        let sample = |id, date, sender_id: &str| Message {
            id,
            date,
            sender_id: sender_id.into(),
            text: format!("fixture {id}"),
            outgoing: false,
            delivery: DeliveryState::Received,
        };
        let items = timeline(&[
            sample(1, 1_700_000_000, "user:1"),
            sample(2, 1_700_000_030, "user:1"),
            sample(3, 1_700_100_000, "user:2"),
        ]);
        assert_eq!(items.len(), 5);
        assert!(matches!(items[0], TimelineItem::Date(_)));
        assert!(matches!(
            items[1],
            TimelineItem::Message {
                grouped: false,
                show_metadata: false,
                ..
            }
        ));
        assert!(matches!(
            items[2],
            TimelineItem::Message {
                grouped: true,
                show_metadata: true,
                ..
            }
        ));
        assert!(matches!(items[3], TimelineItem::Date(_)));
    }

    #[test]
    fn capped_prepend_estimates_scroll_anchor_from_inserted_rows() {
        assert_eq!(estimate_anchor_offset(60_000.0, 1_000, 50), 3_000.0);
        assert_eq!(estimate_anchor_offset(0.0, 0, 50), 0.0);
    }
}
