//! Bounded, UI-independent message window and history pagination state.

use std::collections::{HashMap, HashSet};

pub const PAGE_SIZE: i32 = 50;
pub const MAX_WINDOW_MESSAGES: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChatId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryState {
    Received,
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageDirection {
    Initial,
    Older,
    Newer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: i64,
    pub date: i64,
    pub sender_id: String,
    pub text: String,
    pub outgoing: bool,
    pub delivery: DeliveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryPage {
    pub chat_id: ChatId,
    pub request_id: u64,
    pub messages: Vec<Message>,
    pub has_older: bool,
    pub has_newer: bool,
    pub direction: PageDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryWindow {
    chat_id: Option<ChatId>,
    messages: Vec<Message>,
    pending_request: Option<u64>,
    pending_direction: PageDirection,
    next_request: u64,
    has_older: bool,
    has_newer: bool,
}

impl Default for HistoryWindow {
    fn default() -> Self {
        Self {
            chat_id: None,
            messages: Vec::new(),
            pending_request: None,
            pending_direction: PageDirection::Initial,
            next_request: 1,
            has_older: true,
            has_newer: false,
        }
    }
}

impl HistoryWindow {
    pub fn chat_id(&self) -> Option<ChatId> {
        self.chat_id
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn has_older(&self) -> bool {
        self.has_older
    }

    pub fn has_newer(&self) -> bool {
        self.has_newer
    }

    pub fn is_loading(&self) -> bool {
        self.pending_request.is_some()
    }

    pub fn clear(&mut self) {
        self.chat_id = None;
        self.messages.clear();
        self.pending_request = None;
        self.has_older = true;
        self.has_newer = false;
        self.pending_direction = PageDirection::Initial;
    }

    /// Switches conversations and returns a request ID for the initial page.
    pub fn open_chat(&mut self, chat_id: ChatId) -> u64 {
        self.chat_id = Some(chat_id);
        self.messages.clear();
        self.pending_request = None;
        self.has_older = true;
        self.has_newer = false;
        self.pending_direction = PageDirection::Initial;
        self.start_request()
    }

    /// Returns a request ID only when the current window can page farther back.
    pub fn request_older(&mut self) -> Option<(u64, i64)> {
        if self.pending_request.is_some() || !self.has_older {
            return None;
        }
        let oldest = self.messages.first()?.id;
        self.pending_direction = PageDirection::Older;
        Some((self.start_request(), oldest))
    }

    pub fn request_newer(&mut self) -> Option<(u64, i64)> {
        if self.pending_request.is_some() || !self.has_newer {
            return None;
        }
        let newest = self.messages.last()?.id;
        self.pending_direction = PageDirection::Newer;
        Some((self.start_request(), newest))
    }

    pub fn fail_request(&mut self, chat_id: ChatId, request_id: u64) -> bool {
        if self.chat_id == Some(chat_id) && self.pending_request == Some(request_id) {
            self.pending_request = None;
            true
        } else {
            false
        }
    }

    /// Applies only the active request, sorts by date/ID, deduplicates IDs and
    /// keeps the newest bounded window. Returns whether the UI should refresh.
    pub fn apply_page(&mut self, page: HistoryPage) -> bool {
        if self.chat_id != Some(page.chat_id)
            || self.pending_request != Some(page.request_id)
            || self.pending_direction != page.direction
        {
            return false;
        }
        self.pending_request = None;
        match page.direction {
            PageDirection::Initial => {
                self.has_older = page.has_older;
                self.has_newer = page.has_newer;
            }
            PageDirection::Older => {
                self.has_older = page.has_older;
                self.has_newer = true;
            }
            PageDirection::Newer => {
                self.has_newer = page.has_newer;
                self.has_older = true;
            }
        }
        let mut by_id: HashMap<i64, Message> = self
            .messages
            .drain(..)
            .map(|message| (message.id, message))
            .collect();
        for message in page.messages {
            by_id.insert(message.id, message);
        }
        let mut messages: Vec<_> = by_id.into_values().collect();
        messages.sort_by_key(|message| (message.date, message.id));
        if messages.len() > MAX_WINDOW_MESSAGES {
            let overflow = messages.len() - MAX_WINDOW_MESSAGES;
            match page.direction {
                PageDirection::Older => {
                    messages.truncate(MAX_WINDOW_MESSAGES);
                    self.has_newer = true;
                }
                PageDirection::Newer => {
                    messages.drain(..overflow);
                    self.has_older = true;
                }
                PageDirection::Initial => {
                    messages.drain(..overflow);
                    self.has_older = true;
                }
            }
        }
        self.messages = messages;
        true
    }

    pub fn upsert(&mut self, chat_id: ChatId, message: Message) -> bool {
        if self.chat_id != Some(chat_id) {
            return false;
        }
        if self.messages.len() >= MAX_WINDOW_MESSAGES
            && self.has_newer
            && self
                .messages
                .last()
                .is_some_and(|last| (message.date, message.id) > (last.date, last.id))
        {
            // Keep the older window stable; the newer cursor can fetch this
            // live update when the user returns to the newest edge.
            return false;
        }
        match self
            .messages
            .binary_search_by_key(&(message.date, message.id), |item| (item.date, item.id))
        {
            Ok(index) => self.messages[index] = message,
            Err(index) => self.messages.insert(index, message),
        }
        if self.messages.len() > MAX_WINDOW_MESSAGES {
            self.messages.remove(0);
            self.has_older = true;
        }
        true
    }

    pub fn delete(&mut self, chat_id: ChatId, message_id: i64) -> bool {
        if self.chat_id != Some(chat_id) {
            return false;
        }
        let old_len = self.messages.len();
        self.messages.retain(|message| message.id != message_id);
        old_len != self.messages.len()
    }

    pub fn edit_text(&mut self, chat_id: ChatId, message_id: i64, text: String) -> bool {
        if self.chat_id != Some(chat_id) {
            return false;
        }
        if let Some(message) = self
            .messages
            .iter_mut()
            .find(|message| message.id == message_id)
        {
            message.text = text;
            true
        } else {
            false
        }
    }

    pub fn update_delivery(
        &mut self,
        chat_id: ChatId,
        message_id: i64,
        delivery: DeliveryState,
    ) -> bool {
        if self.chat_id != Some(chat_id) {
            return false;
        }
        if let Some(message) = self
            .messages
            .iter_mut()
            .find(|message| message.id == message_id)
        {
            message.delivery = delivery;
            true
        } else {
            false
        }
    }

    fn start_request(&mut self) -> u64 {
        let id = self.next_request;
        self.next_request = self.next_request.wrapping_add(1).max(1);
        self.pending_request = Some(id);
        id
    }
}

/// Stable ID de-duplication helper used by list models and tests.
pub fn unique_ids(messages: &[Message]) -> HashSet<i64> {
    messages.iter().map(|message| message.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(id: i64, date: i64) -> Message {
        Message {
            id,
            date,
            sender_id: "user:synthetic".into(),
            text: format!("synthetic {id}"),
            outgoing: false,
            delivery: DeliveryState::Received,
        }
    }

    #[test]
    fn pages_are_sorted_and_deduplicated() {
        let mut history = HistoryWindow::default();
        let request_id = history.open_chat(ChatId(7));
        assert!(history.apply_page(HistoryPage {
            chat_id: ChatId(7),
            request_id,
            messages: vec![message(3, 3), message(2, 2), message(2, 2)],
            has_older: true,
            has_newer: false,
            direction: PageDirection::Initial,
        }));
        assert_eq!(
            history.messages().iter().map(|m| m.id).collect::<Vec<_>>(),
            [2, 3]
        );
        assert_eq!(unique_ids(history.messages()).len(), 2);
    }

    #[test]
    fn stale_page_from_previous_chat_is_ignored() {
        let mut history = HistoryWindow::default();
        let old_request = history.open_chat(ChatId(1));
        history.open_chat(ChatId(2));
        assert!(!history.apply_page(HistoryPage {
            chat_id: ChatId(1),
            request_id: old_request,
            messages: vec![message(1, 1)],
            has_older: false,
            has_newer: false,
            direction: PageDirection::Initial,
        }));
        assert_eq!(history.chat_id(), Some(ChatId(2)));
        assert!(history.messages().is_empty());
    }

    #[test]
    fn older_request_uses_oldest_loaded_message_and_has_single_flight() {
        let mut history = HistoryWindow::default();
        let first = history.open_chat(ChatId(9));
        history.apply_page(HistoryPage {
            chat_id: ChatId(9),
            request_id: first,
            messages: vec![message(11, 11), message(10, 10)],
            has_older: true,
            has_newer: false,
            direction: PageDirection::Initial,
        });
        let (request, from_id) = history.request_older().unwrap();
        assert_eq!(from_id, 10);
        assert!(history.request_older().is_none());
        assert!(history.fail_request(ChatId(9), request));
        assert!(!history.is_loading());
    }

    #[test]
    fn message_window_is_bounded_and_live_mutations_are_scoped() {
        let mut history = HistoryWindow::default();
        let request = history.open_chat(ChatId(4));
        let messages = (0..MAX_WINDOW_MESSAGES as i64 + 1)
            .map(|id| message(id, id))
            .collect();
        history.apply_page(HistoryPage {
            chat_id: ChatId(4),
            request_id: request,
            messages,
            has_older: true,
            has_newer: false,
            direction: PageDirection::Initial,
        });
        assert_eq!(history.messages().len(), MAX_WINDOW_MESSAGES);
        assert!(!history.delete(ChatId(5), 1));
        assert!(history.upsert(ChatId(4), message(2_000, 2_000)));
        assert_eq!(history.messages().len(), MAX_WINDOW_MESSAGES);
    }

    #[test]
    fn bounded_window_moves_older_and_newer_without_losing_the_cursors() {
        let mut history = HistoryWindow::default();
        let initial = history.open_chat(ChatId(12));
        history.apply_page(HistoryPage {
            chat_id: ChatId(12),
            request_id: initial,
            messages: (1_000..2_000).map(|id| message(id, id)).collect(),
            has_older: true,
            has_newer: false,
            direction: PageDirection::Initial,
        });
        let (older_id, from_id) = history.request_older().unwrap();
        assert_eq!(from_id, 1_000);
        history.apply_page(HistoryPage {
            chat_id: ChatId(12),
            request_id: older_id,
            messages: (950..1_000).map(|id| message(id, id)).collect(),
            has_older: true,
            has_newer: true,
            direction: PageDirection::Older,
        });
        assert_eq!(history.messages().len(), MAX_WINDOW_MESSAGES);
        assert_eq!(history.messages().first().unwrap().id, 950);
        assert_eq!(history.messages().last().unwrap().id, 1_949);
        assert!(history.has_newer());
        assert!(!history.upsert(ChatId(12), message(3_000, 3_000)));
        assert_eq!(history.messages().first().unwrap().id, 950);

        let (newer_id, from_id) = history.request_newer().unwrap();
        assert_eq!(from_id, 1_949);
        history.apply_page(HistoryPage {
            chat_id: ChatId(12),
            request_id: newer_id,
            messages: (1_949..1_999).map(|id| message(id, id)).collect(),
            has_older: true,
            has_newer: true,
            direction: PageDirection::Newer,
        });
        assert_eq!(history.messages().len(), MAX_WINDOW_MESSAGES);
        assert_eq!(history.messages().first().unwrap().id, 999);
        assert!(history.has_older());
        assert!(history.has_newer());
    }
}
