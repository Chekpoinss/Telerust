//! Typed application events. The initial shell intentionally has no event loop.

/// An opaque identifier for a conversation, scoped to an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChatId(pub i64);

/// Events crossing from a backend adapter into application state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendEvent {
    AuthorizationChanged(AuthorizationState),
    ChatChanged(ChatId),
    MessageChanged { chat: ChatId, message_id: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationState {
    WaitingForParameters,
    WaitingForPhoneNumber,
    WaitingForCode,
    WaitingForPassword,
    WaitingForQrConfirmation,
    Ready,
    LoggingOut,
    Closed,
}
