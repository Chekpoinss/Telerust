//! UI-independent domain boundary. No Telegram protocol or persistence is
//! implemented here yet; this crate defines where those adapters will connect.

pub mod events;
pub mod features;
pub mod history;
pub mod settings;
