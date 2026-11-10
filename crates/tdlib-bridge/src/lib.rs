//! Asynchronous TDLib adapter using its current JSON C ABI.
//! All unsafe code and native pointers are contained in this crate.

use async_channel::{Receiver, Sender};
use libloading::Library;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    os::raw::c_char,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use telerust_client_core::{
    events::AuthorizationState,
    history::{MediaFile, MessageKind, PageDirection, PAGE_SIZE},
    settings::Settings,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSummary {
    pub id: i64,
    pub title: String,
    pub preview: String,
    pub sort_date: i64,
    pub is_channel: bool,
    pub photo_file_id: Option<i32>,
    pub photo_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageView {
    pub id: i64,
    pub date: i64,
    pub sender_id: String,
    pub text: String,
    pub kind: MessageKind,
    pub outgoing: bool,
    pub delivery: DeliveryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryState {
    Received,
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthPrompt {
    PhoneNumber,
    Code,
    Password,
    EmailAddress,
    EmailCode,
    Registration,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum BackendEvent {
    Authorization(AuthorizationState),
    QrLink(String),
    Prompt(AuthPrompt),
    Chats(Vec<ChatSummary>),
    History {
        chat_id: i64,
        request_id: u64,
        messages: Vec<MessageView>,
        has_older: bool,
        has_newer: bool,
        direction: PageDirection,
    },
    HistoryError {
        chat_id: i64,
        request_id: u64,
        message: String,
    },
    NewMessage {
        chat_id: i64,
        message: MessageView,
    },
    MessageEdited {
        chat_id: i64,
        message: MessageView,
    },
    MessageDeleted {
        chat_id: i64,
        message_id: i64,
    },
    MessageDelivery {
        chat_id: i64,
        message_id: i64,
        delivery: DeliveryState,
    },
    FileUpdated {
        file_id: i32,
        local_path: Option<String>,
        downloaded: bool,
        downloading: bool,
        downloaded_size: i64,
    },
    FileDownloadFailed {
        file_id: i32,
        message: String,
    },
    Status(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum BackendCommand {
    RequestQr,
    UsePhone,
    Logout,
    PhoneNumber(String),
    Code(String),
    Password(String),
    EmailAddress(String),
    EmailCode(String),
    OpenChat {
        chat_id: i64,
        request_id: u64,
    },
    LoadOlder {
        chat_id: i64,
        request_id: u64,
        from_message_id: i64,
    },
    LoadNewer {
        chat_id: i64,
        request_id: u64,
        from_message_id: i64,
    },
    SendText {
        chat_id: i64,
        text: String,
    },
    DownloadFile {
        file_id: i32,
    },
    Shutdown,
}

pub fn spawn(settings: Settings) -> (Sender<BackendCommand>, Receiver<BackendEvent>) {
    let (command_tx, command_rx) = async_channel::bounded(32);
    let (event_tx, event_rx) = async_channel::bounded(256);
    let error_tx = event_tx.clone();
    if thread::Builder::new()
        .name("telerust-tdlib".into())
        .spawn(move || run(settings, command_rx, event_tx))
        .is_err()
    {
        let _ = error_tx.try_send(BackendEvent::Error(
            "Could not start the TDLib worker thread.".into(),
        ));
    }
    (command_tx, event_rx)
}

/// Load TDLib's current JSON C ABI and receive the initial auth state without
/// configuring TDLib parameters or starting a network connection.
pub fn probe_tdlib_library(path: &Path) -> Result<(), &'static str> {
    let mut client = TdClient::load(Some(path)).map_err(|error| error.as_str())?;
    client.send(&json!({"@type":"getAuthorizationState"}));
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut initial_state = false;
    while Instant::now() < deadline {
        if let Some(update) = client.receive(0.1) {
            if update.get("@type").and_then(Value::as_str) == Some("updateAuthorizationState")
                && update
                    .pointer("/authorization_state/@type")
                    .and_then(Value::as_str)
                    == Some("authorizationStateWaitTdlibParameters")
            {
                initial_state = true;
                break;
            }
        }
    }
    close_client(&mut client);
    if initial_state {
        Ok(())
    } else {
        Err("TDLib did not return its initial authorization state")
    }
}

fn run(settings: Settings, commands: Receiver<BackendCommand>, events: Sender<BackendEvent>) {
    let mut prefer_qr = true;
    loop {
        match run_inner(settings.clone(), &commands, &events, &mut prefer_qr) {
            Ok(true) => continue,
            Ok(false) => break,
            Err(error) => {
                let _ = events.send_blocking(BackendEvent::Error(error.to_string()));
                break;
            }
        }
    }
}

#[derive(Debug)]
enum BackendError {
    MissingLibrary,
    UnloadableLibrary,
    MissingSymbol,
    Keyring,
    Storage,
}

impl BackendError {
    fn as_str(&self) -> &'static str {
        match self {
            Self::MissingLibrary => "TDLib runtime is missing. Install a compatible libtdjson.so, or place it under the Telerust data directory's lib/ folder, then restart.",
            Self::UnloadableLibrary => {
                "TDLib was found but could not be loaded. Check the library architecture and dynamic dependencies."
            }
            Self::MissingSymbol => "TDLib loaded, but its JSON C ABI is incomplete or incompatible. Install a matching official TDLib build.",
            Self::Keyring => "GNOME Keyring is unavailable or locked; unlock it and restart.",
            Self::Storage => "The private session directory could not be prepared.",
        }
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

fn run_inner(
    settings: Settings,
    commands: &Receiver<BackendCommand>,
    events: &Sender<BackendEvent>,
    prefer_qr: &mut bool,
) -> Result<bool, BackendError> {
    let environment = if settings.use_test_dc { "test" } else { "prod" };
    let db_dir = data_path().join(format!("tdlib-{environment}"));
    std::fs::create_dir_all(&db_dir).map_err(|_| BackendError::Storage)?;
    let files_dir = db_dir.join("files");
    std::fs::create_dir_all(&files_dir).map_err(|_| BackendError::Storage)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&db_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| BackendError::Storage)?;
        std::fs::set_permissions(&files_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| BackendError::Storage)?;
    }
    let mut client = TdClient::load(settings.tdlib_library.as_deref())?;
    let key = get_database_key(settings.api_id, environment)?;
    if let Some(proxy) = gnome_proxy_request() {
        client.send(&proxy);
        let _ = events.send_blocking(BackendEvent::Status(
            "Connecting to Telegram through the GNOME system proxy…".into(),
        ));
    }
    client.send(&json!({"@type":"getAuthorizationState"}));

    let mut chats: HashMap<i64, ChatSummary> = HashMap::new();
    let mut selected_chat = None;
    let mut pending_history: HashMap<u64, (i64, i64, PageDirection)> = HashMap::new();
    let mut requested_files = HashSet::new();
    let mut closed = false;
    let mut restart = false;
    while !closed {
        if let Some(update) = client.receive(0.2) {
            handle_update(
                &mut client,
                update,
                UpdateContext {
                    settings: &settings,
                    db_dir: &db_dir,
                    key: &key,
                    chats: &mut chats,
                    selected_chat: &mut selected_chat,
                    pending_history: &mut pending_history,
                    requested_files: &mut requested_files,
                    events,
                    closed: &mut closed,
                    prefer_qr,
                },
            );
        }
        while let Ok(command) = commands.try_recv() {
            match command {
                BackendCommand::RequestQr => {
                    *prefer_qr = true;
                    client
                        .send(&json!({"@type":"requestQrCodeAuthentication","other_user_ids":[]}));
                }
                BackendCommand::UsePhone => {
                    *prefer_qr = false;
                    client.send(&json!({"@type":"destroy"}));
                    restart = true;
                }
                BackendCommand::Logout => {
                    *prefer_qr = true;
                    client.send(&json!({"@type":"logOut"}));
                    restart = true;
                }
                BackendCommand::PhoneNumber(phone) => {
                    client.send(
                        &json!({"@type":"setAuthenticationPhoneNumber","phone_number":phone}),
                    );
                }
                BackendCommand::Code(code) => {
                    client.send(&json!({"@type":"checkAuthenticationCode","code":code}));
                }
                BackendCommand::Password(password) => {
                    client
                        .send(&json!({"@type":"checkAuthenticationPassword","password":password}));
                }
                BackendCommand::EmailAddress(email) => {
                    client.send(
                        &json!({"@type":"setAuthenticationEmailAddress","email_address":email}),
                    );
                }
                BackendCommand::EmailCode(code) => {
                    client.send(&json!({"@type":"checkAuthenticationEmailCode","code":{"@type":"emailAddressAuthenticationCode","code":code}}));
                }
                BackendCommand::OpenChat {
                    chat_id,
                    request_id,
                } => {
                    if chats.get(&chat_id).is_some_and(|chat| chat.is_channel) {
                        let _ = events.send_blocking(BackendEvent::Error(
                            "Channel chats are disabled until sponsored message support is implemented.".into(),
                        ));
                        continue;
                    }
                    selected_chat = Some(chat_id);
                    pending_history.clear();
                    pending_history.insert(request_id, (chat_id, 0, PageDirection::Initial));
                    client.send(&history_query(
                        chat_id,
                        0,
                        request_id,
                        PageDirection::Initial,
                    ));
                }
                BackendCommand::LoadOlder {
                    chat_id,
                    request_id,
                    from_message_id,
                } => {
                    if selected_chat == Some(chat_id) && from_message_id > 0 {
                        pending_history
                            .insert(request_id, (chat_id, from_message_id, PageDirection::Older));
                        client.send(&history_query(
                            chat_id,
                            from_message_id,
                            request_id,
                            PageDirection::Older,
                        ));
                    }
                }
                BackendCommand::LoadNewer {
                    chat_id,
                    request_id,
                    from_message_id,
                } => {
                    if selected_chat == Some(chat_id) && from_message_id > 0 {
                        pending_history
                            .insert(request_id, (chat_id, from_message_id, PageDirection::Newer));
                        client.send(&history_query(
                            chat_id,
                            from_message_id,
                            request_id,
                            PageDirection::Newer,
                        ));
                    }
                }
                BackendCommand::SendText { chat_id, text } => {
                    if !text.trim().is_empty()
                        && !chats.get(&chat_id).is_some_and(|chat| chat.is_channel)
                    {
                        client.send(&json!({
                            "@type":"sendMessage", "chat_id":chat_id,
                            "input_message_content": {
                                "@type":"inputMessageText",
                                "text":{"@type":"formattedText","text":text,"entities":[]},
                                "clear_draft":true
                            }
                        }));
                    }
                }
                BackendCommand::DownloadFile { file_id } => {
                    if file_id > 0 && requested_files.insert(file_id) {
                        client.send(&download_file_query(file_id));
                    }
                }
                BackendCommand::Shutdown => {
                    restart = false;
                    close_client(&mut client);
                    closed = true;
                    break;
                }
            }
        }
        if commands.is_closed() {
            restart = false;
            close_client(&mut client);
            closed = true;
        }
    }
    if restart {
        let entry = keyring::Entry::new(
            "org.telerust.Session",
            &format!("tdlib-db-key-{}-{environment}", settings.api_id),
        )
        .map_err(|_| BackendError::Keyring)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => {
                let _ = events.send_blocking(BackendEvent::Error(
                    "TDLib logged out, but its database key could not be removed from Secret Service.".into(),
                ));
                return Err(BackendError::Keyring);
            }
        }
    }
    Ok(restart)
}

fn close_client(client: &mut TdClient) {
    client.send(&json!({"@type":"close"}));
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Some(update) = client.receive(0.1) {
            if update.get("@type").and_then(Value::as_str) == Some("updateAuthorizationState")
                && update
                    .pointer("/authorization_state/@type")
                    .and_then(Value::as_str)
                    == Some("authorizationStateClosed")
            {
                break;
            }
        }
    }
}

struct UpdateContext<'a> {
    settings: &'a Settings,
    db_dir: &'a Path,
    key: &'a str,
    chats: &'a mut HashMap<i64, ChatSummary>,
    selected_chat: &'a mut Option<i64>,
    pending_history: &'a mut HashMap<u64, (i64, i64, PageDirection)>,
    requested_files: &'a mut HashSet<i32>,
    events: &'a Sender<BackendEvent>,
    closed: &'a mut bool,
    prefer_qr: &'a mut bool,
}

fn handle_update(client: &mut TdClient, update: Value, context: UpdateContext<'_>) {
    let UpdateContext {
        settings,
        db_dir,
        key,
        chats,
        selected_chat,
        pending_history,
        requested_files,
        events,
        closed,
        prefer_qr,
    } = context;
    match update.get("@type").and_then(Value::as_str).unwrap_or("") {
        "updateConnectionState" => {
            if let Some(status) = connection_status(
                update
                    .pointer("/state/@type")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ) {
                let _ = events.send_blocking(BackendEvent::Status(status.into()));
            }
        }
        "updateAuthorizationState" => {
            let state = update
                .pointer("/authorization_state/@type")
                .and_then(Value::as_str)
                .unwrap_or("");
            match authorization_state_from_tdlib(state) {
                AuthorizationState::WaitingForQrConfirmation => {
                    let link = update
                        .pointer("/authorization_state/link")
                        .and_then(Value::as_str);
                    if let Some(link) = link.filter(|link| link.starts_with("tg://login?token=")) {
                        let _ = events.send_blocking(BackendEvent::QrLink(link.to_owned()));
                        let _ = events.send_blocking(BackendEvent::Authorization(
                            AuthorizationState::WaitingForQrConfirmation,
                        ));
                    } else {
                        let _ = events.send_blocking(BackendEvent::Error(
                            "Telegram returned an invalid QR login link.".into(),
                        ));
                    }
                    return;
                }
                AuthorizationState::LoggingOut => {
                    let _ = events
                        .send_blocking(BackendEvent::Authorization(AuthorizationState::LoggingOut));
                    return;
                }
                _ => {}
            }
            let (mapped, prompt) = match state {
                "authorizationStateWaitTdlibParameters" => {
                    let _ = events.send_blocking(BackendEvent::Status(
                        "Initializing TDLib with the saved application credentials…".into(),
                    ));
                    client.send(&json!({
                        "@type":"setTdlibParameters", "use_test_dc":settings.use_test_dc,
                        "database_directory":db_dir.to_string_lossy(),
                        "files_directory":db_dir.join("files").to_string_lossy(),
                        "database_encryption_key":key,
                        "use_file_database":true, "use_chat_info_database":true,
                        "use_message_database":true, "use_secret_chats":false,
                        "api_id":settings.api_id, "api_hash":settings.api_hash,
                        "system_language_code":"en", "device_model":"Fedora GNOME Desktop",
                        "system_version":"Linux", "application_version":env!("CARGO_PKG_VERSION"),
                        "enable_storage_optimizer":true
                    }));
                    (AuthorizationState::WaitingForParameters, None)
                }
                "authorizationStateWaitEncryptionKey" => {
                    let _ = events.send_blocking(BackendEvent::Status(
                        "Opening the encrypted local Telegram session…".into(),
                    ));
                    client
                        .send(&json!({"@type":"checkDatabaseEncryptionKey","encryption_key":key}));
                    (AuthorizationState::WaitingForParameters, None)
                }
                "authorizationStateWaitPhoneNumber" => {
                    if *prefer_qr {
                        let _ = events.send_blocking(BackendEvent::Status(
                            "TDLib is ready. Requesting a live QR code from Telegram…".into(),
                        ));
                        client.send(
                            &json!({"@type":"requestQrCodeAuthentication","other_user_ids":[]}),
                        );
                        return;
                    }
                    let _ = events.send_blocking(BackendEvent::Status(
                        "Enter your phone number with country code.".into(),
                    ));
                    (
                        AuthorizationState::WaitingForPhoneNumber,
                        Some(AuthPrompt::PhoneNumber),
                    )
                }
                "authorizationStateWaitCode" => {
                    let _ = events.send_blocking(BackendEvent::Status(
                        "Enter the login code sent by Telegram.".into(),
                    ));
                    (AuthorizationState::WaitingForCode, Some(AuthPrompt::Code))
                }
                "authorizationStateWaitPassword" => {
                    let _ = events.send_blocking(BackendEvent::Status(
                        "Enter your Telegram two-step verification password.".into(),
                    ));
                    (
                        AuthorizationState::WaitingForPassword,
                        Some(AuthPrompt::Password),
                    )
                }
                "authorizationStateWaitEmailAddress" => (
                    AuthorizationState::WaitingForPhoneNumber,
                    Some(AuthPrompt::EmailAddress),
                ),
                "authorizationStateWaitEmailCode" => (
                    AuthorizationState::WaitingForCode,
                    Some(AuthPrompt::EmailCode),
                ),
                "authorizationStateWaitRegistration" => (
                    AuthorizationState::WaitingForPhoneNumber,
                    Some(AuthPrompt::Registration),
                ),
                "authorizationStateReady" => {
                    let _ = events
                        .send_blocking(BackendEvent::Authorization(AuthorizationState::Ready));
                    client.send(&json!({"@type":"loadChats","chat_list":{"@type":"chatListMain"},"limit":100}));
                    return;
                }
                "authorizationStateClosed" => {
                    *closed = true;
                    (AuthorizationState::Closed, None)
                }
                other => (
                    AuthorizationState::WaitingForParameters,
                    Some(AuthPrompt::Other(other.to_owned())),
                ),
            };
            let _ = events.send_blocking(BackendEvent::Authorization(mapped));
            if let Some(prompt) = prompt {
                let _ = events.send_blocking(BackendEvent::Prompt(prompt));
            }
        }
        "updateNewChat" => {
            if let Some(chat) = update.get("chat") {
                upsert_chat(chat, chats);
                emit_chats(chats, events);
            }
        }
        "updateChatTitle" | "updateChatLastMessage" | "updateChatPosition" | "updateChatPhoto" => {
            let id = update.get("chat_id").and_then(Value::as_i64);
            if let Some(id) = id {
                let chat = chats.entry(id).or_insert_with(|| ChatSummary {
                    id,
                    title: format!("Chat {id}"),
                    preview: String::new(),
                    sort_date: 0,
                    is_channel: false,
                    photo_file_id: None,
                    photo_path: None,
                });
                if update.get("@type").and_then(Value::as_str) == Some("updateChatTitle") {
                    if let Some(title) = update.get("title").and_then(Value::as_str) {
                        chat.title = title.to_owned();
                    }
                }
                if update.get("@type").and_then(Value::as_str) == Some("updateChatLastMessage") {
                    if let Some(message) = update.get("last_message") {
                        chat.preview = message_text(message);
                        chat.sort_date = message.get("date").and_then(Value::as_i64).unwrap_or(0);
                    }
                }
                if update.get("@type").and_then(Value::as_str) == Some("updateChatPhoto") {
                    update_chat_photo(chat, &update["photo"]);
                }
                emit_chats(chats, events);
            }
        }
        "updateNewMessage" => {
            if let Some(message) = update.get("message") {
                let chat_id = message.get("chat_id").and_then(Value::as_i64).unwrap_or(0);
                let msg = message_view(message);
                if let Some(chat) = chats.get_mut(&chat_id) {
                    chat.preview = msg.text.clone();
                    chat.sort_date = message.get("date").and_then(Value::as_i64).unwrap_or(0);
                }
                emit_chats(chats, events);
                if Some(chat_id) == *selected_chat {
                    let _ = events.send_blocking(BackendEvent::NewMessage {
                        chat_id,
                        message: msg,
                    });
                }
            }
        }
        "updateMessageContent" => {
            if let (Some(chat_id), Some(message_id)) = (
                update.get("chat_id").and_then(Value::as_i64),
                update.get("message_id").and_then(Value::as_i64),
            ) {
                let (text, kind) = message_content(&update["new_content"]);
                let message = MessageView {
                    id: message_id,
                    date: 0,
                    sender_id: String::new(),
                    text,
                    kind,
                    outgoing: false,
                    delivery: DeliveryState::Received,
                };
                let _ = events.send_blocking(BackendEvent::MessageEdited { chat_id, message });
            }
        }
        "updateMessageSendSucceeded" => {
            if let (Some(old_message_id), Some(message)) = (
                update.get("old_message_id").and_then(Value::as_i64),
                update.get("message"),
            ) {
                let chat_id = message
                    .get("chat_id")
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                let _ = events.send_blocking(BackendEvent::MessageDeleted {
                    chat_id,
                    message_id: old_message_id,
                });
                let _ = events.send_blocking(BackendEvent::NewMessage {
                    chat_id,
                    message: message_view(message),
                });
            }
        }
        "updateDeleteMessages" => {
            if let (Some(chat_id), Some(ids)) = (
                update.get("chat_id").and_then(Value::as_i64),
                update.get("message_ids").and_then(Value::as_array),
            ) {
                for message_id in ids.iter().filter_map(Value::as_i64) {
                    let _ = events.send_blocking(BackendEvent::MessageDeleted {
                        chat_id,
                        message_id,
                    });
                }
            }
        }
        "messages" => {
            if let Some(messages) = update.get("messages").and_then(Value::as_array) {
                let request_id = update.get("@extra").and_then(Value::as_u64);
                if let Some((chat_id, from_id, direction)) =
                    request_id.and_then(|id| pending_history.remove(&id))
                {
                    let views: Vec<_> = messages.iter().map(message_view).collect();
                    let has_older = match direction {
                        PageDirection::Initial => !views.is_empty(),
                        PageDirection::Older => views.iter().any(|message| message.id < from_id),
                        PageDirection::Newer => true,
                    };
                    let has_newer = match direction {
                        PageDirection::Initial => false,
                        PageDirection::Older => true,
                        PageDirection::Newer => views.iter().any(|message| message.id > from_id),
                    };
                    let _ = events.send_blocking(BackendEvent::History {
                        chat_id,
                        request_id: request_id.unwrap_or_default(),
                        messages: views,
                        has_older,
                        has_newer,
                        direction,
                    });
                }
            }
        }
        "error" => {
            let code = update.get("code").and_then(Value::as_i64).unwrap_or(0);
            let error_name = update.get("message").and_then(Value::as_str).unwrap_or("");
            let message = tdlib_error_message(code, error_name);
            if let Some(file_id) = update
                .get("@extra")
                .and_then(Value::as_str)
                .and_then(|extra| extra.strip_prefix("telerust-file-"))
                .and_then(|id| id.parse::<i32>().ok())
            {
                requested_files.remove(&file_id);
                let _ = events.send_blocking(BackendEvent::FileDownloadFailed {
                    file_id,
                    message: format!("File download failed (Telegram code {code}). You can retry."),
                });
                return;
            }
            if let Some(request_id) = update.get("@extra").and_then(Value::as_u64) {
                if let Some((chat_id, _, _)) = pending_history.remove(&request_id) {
                    let _ = events.send_blocking(BackendEvent::HistoryError {
                        chat_id,
                        request_id,
                        message,
                    });
                    return;
                }
            }
            let _ = events.send_blocking(BackendEvent::Error(message));
        }
        "updateMessageSendFailed" => {
            let code = update
                .pointer("/error_code")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            if let (Some(message), Some(old_message_id)) = (
                update.get("message"),
                update.get("old_message_id").and_then(Value::as_i64),
            ) {
                let chat_id = message
                    .get("chat_id")
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                let _ = events.send_blocking(BackendEvent::MessageDelivery {
                    chat_id,
                    message_id: old_message_id,
                    delivery: DeliveryState::Failed,
                });
            } else {
                let _ = events.send_blocking(BackendEvent::Error(format!(
                    "Message was not sent (code {code})."
                )));
            }
        }
        "updateFile" => {
            if let Some(file) = update.get("file") {
                if let Some(file_id) = file.get("id").and_then(Value::as_i64) {
                    let local = &file["local"];
                    let downloaded = local
                        .get("is_downloading_completed")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let local_path = local
                        .get("path")
                        .and_then(Value::as_str)
                        .filter(|path| !path.is_empty())
                        .map(str::to_owned);
                    let downloading = local
                        .get("is_downloading_active")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let downloaded_size = local
                        .get("downloaded_prefix_size")
                        .and_then(Value::as_i64)
                        .or_else(|| local.get("downloaded_size").and_then(Value::as_i64))
                        .unwrap_or(0);
                    if downloaded
                        || !local
                            .get("is_downloading_active")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                    {
                        requested_files.remove(&(file_id as i32));
                    }
                    let _ = events.send_blocking(BackendEvent::FileUpdated {
                        file_id: file_id as i32,
                        local_path: local_path.clone(),
                        downloaded,
                        downloading,
                        downloaded_size,
                    });
                    let changed_chat_photo = chats.values_mut().any(|chat| {
                        if chat.photo_file_id == Some(file_id as i32) {
                            chat.photo_path = local_path.clone().filter(|_| downloaded);
                            true
                        } else {
                            false
                        }
                    });
                    if changed_chat_photo {
                        emit_chats(chats, events);
                    }
                }
            }
        }
        _ => {}
    }
}

fn history_query(
    chat_id: i64,
    from_message_id: i64,
    request_id: u64,
    direction: PageDirection,
) -> Value {
    json!({
        "@type":"getChatHistory", "chat_id":chat_id,
        "from_message_id":from_message_id,
        "offset":if direction == PageDirection::Newer { 1 - PAGE_SIZE } else { 0 },
        "limit":PAGE_SIZE, "only_local":false, "@extra":request_id
    })
}

fn download_file_query(file_id: i32) -> Value {
    json!({
        "@type":"downloadFile", "file_id":file_id,
        "priority":16, "offset":0, "limit":0, "synchronous":false,
        "@extra":format!("telerust-file-{file_id}")
    })
}

fn tdlib_error_message(code: i64, error_name: &str) -> String {
    let known = match error_name {
        "API_ID_INVALID" => Some(
            "Telegram rejected the saved API credentials. Check them in Telerust settings; their values are hidden.",
        ),
        "API_ID_PUBLISHED_FLOOD" => Some(
            "Telegram restricted this API application ID. Use credentials issued for your own application.",
        ),
        "PHONE_NUMBER_INVALID" => Some("That phone number was rejected. Check the country code and try again."),
        "PHONE_CODE_INVALID" => Some("That login code was rejected. Check the latest code and try again."),
        "PHONE_CODE_EXPIRED" => Some("That login code expired. Request a new code and try again."),
        "PASSWORD_HASH_INVALID" => Some("That two-step verification password was rejected. Try again."),
        "FLOOD_WAIT" | "FLOOD_PREMIUM_WAIT" => Some("Telegram is rate limiting sign-in. Wait before trying again."),
        _ => None,
    };
    known.map(str::to_owned).unwrap_or_else(|| {
        format!("Telegram request failed (code {code}). Check the connection and retry.")
    })
}

fn connection_status(state: &str) -> Option<&'static str> {
    match state {
        "connectionStateWaitingForNetwork" => Some(
            "No network is available. Check GNOME network and proxy settings, then wait for reconnect.",
        ),
        "connectionStateConnectingToProxy" => Some("Connecting to Telegram through the configured proxy…"),
        "connectionStateConnecting" => Some("Connecting to Telegram…"),
        "connectionStateUpdating" => Some("Updating the Telegram connection…"),
        "connectionStateReady" => Some("Connected to Telegram."),
        _ => None,
    }
}

fn authorization_state_from_tdlib(state: &str) -> AuthorizationState {
    match state {
        "authorizationStateWaitTdlibParameters" | "authorizationStateWaitEncryptionKey" => {
            AuthorizationState::WaitingForParameters
        }
        "authorizationStateWaitPhoneNumber"
        | "authorizationStateWaitEmailAddress"
        | "authorizationStateWaitRegistration" => AuthorizationState::WaitingForPhoneNumber,
        "authorizationStateWaitCode" | "authorizationStateWaitEmailCode" => {
            AuthorizationState::WaitingForCode
        }
        "authorizationStateWaitPassword" => AuthorizationState::WaitingForPassword,
        "authorizationStateWaitOtherDeviceConfirmation" => {
            AuthorizationState::WaitingForQrConfirmation
        }
        "authorizationStateReady" => AuthorizationState::Ready,
        "authorizationStateLoggingOut" => AuthorizationState::LoggingOut,
        "authorizationStateClosed" => AuthorizationState::Closed,
        _ => AuthorizationState::WaitingForParameters,
    }
}

fn upsert_chat(value: &Value, chats: &mut HashMap<i64, ChatSummary>) {
    let Some(id) = value.get("id").and_then(Value::as_i64) else {
        return;
    };
    let last = value.get("last_message");
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Chat")
        .to_owned();
    let (photo_file_id, photo_path) = chat_photo_fields(&value["photo"]);
    let is_channel = value.pointer("/type/@type").and_then(Value::as_str)
        == Some("chatTypeSupergroup")
        && value
            .pointer("/type/is_channel")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let item = chats.entry(id).or_insert(ChatSummary {
        id,
        title: title.clone(),
        preview: String::new(),
        sort_date: 0,
        is_channel,
        photo_file_id,
        photo_path: photo_path.clone(),
    });
    item.title = title;
    item.is_channel = is_channel;
    item.photo_file_id = photo_file_id;
    item.photo_path = photo_path;
    if let Some(last) = last {
        item.preview = message_text(last);
        item.sort_date = last.get("date").and_then(Value::as_i64).unwrap_or(0);
    }
}

fn chat_photo_fields(photo: &Value) -> (Option<i32>, Option<String>) {
    let file_id = photo
        .pointer("/small/id")
        .and_then(Value::as_i64)
        .map(|id| id as i32);
    let path = photo
        .pointer("/small/local/path")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .map(str::to_owned);
    (file_id, path)
}

fn update_chat_photo(chat: &mut ChatSummary, photo: &Value) {
    (chat.photo_file_id, chat.photo_path) = chat_photo_fields(photo);
}

fn emit_chats(chats: &HashMap<i64, ChatSummary>, events: &Sender<BackendEvent>) {
    let mut chats: Vec<_> = chats
        .values()
        .filter(|chat| !chat.is_channel)
        .cloned()
        .collect();
    chats.sort_by(|a, b| b.sort_date.cmp(&a.sort_date).then_with(|| b.id.cmp(&a.id)));
    let _ = events.send_blocking(BackendEvent::Chats(chats));
}

fn message_view(value: &Value) -> MessageView {
    let (text, kind) = message_content(&value["content"]);
    MessageView {
        id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
        date: value
            .get("date")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        sender_id: value
            .pointer("/sender_id/user_id")
            .and_then(Value::as_i64)
            .map(|id| format!("user:{id}"))
            .or_else(|| {
                value
                    .pointer("/sender_id/chat_id")
                    .and_then(Value::as_i64)
                    .map(|id| format!("chat:{id}"))
            })
            .unwrap_or_default(),
        text,
        kind,
        outgoing: value
            .get("is_outgoing")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        delivery: match value
            .pointer("/sending_state/@type")
            .and_then(Value::as_str)
        {
            Some("messageSendingStatePending") => DeliveryState::Sending,
            Some("messageSendingStateFailed") => DeliveryState::Failed,
            _ if value
                .get("is_outgoing")
                .and_then(Value::as_bool)
                .unwrap_or(false) =>
            {
                DeliveryState::Sent
            }
            _ => DeliveryState::Received,
        },
    }
}

fn message_text(value: &Value) -> String {
    message_content(&value["content"]).0
}

fn message_content(content: &Value) -> (String, MessageKind) {
    let kind_name = content
        .get("@type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let caption = content
        .pointer("/caption/text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    match kind_name {
        "messageText" => {
            let text = content
                .pointer("/text/text")
                .and_then(Value::as_str)
                .unwrap_or("");
            if text.trim().is_empty() {
                (
                    "Empty text message".into(),
                    MessageKind::Unsupported("Empty text message".into()),
                )
            } else {
                (text.to_owned(), MessageKind::Text)
            }
        }
        "messagePhoto" => {
            let sizes = content.pointer("/photo/sizes").and_then(Value::as_array);
            let chosen = sizes.and_then(|sizes| {
                let candidates: Vec<_> = sizes
                    .iter()
                    .filter_map(|size| {
                        let file = size.get("photo")?;
                        let width = size.get("width").and_then(Value::as_i64).unwrap_or(0) as i32;
                        let height = size.get("height").and_then(Value::as_i64).unwrap_or(0) as i32;
                        Some((file, width, height, width as i64 * height as i64))
                    })
                    .collect();
                candidates
                    .iter()
                    .filter(|(_, width, height, _)| *width >= 240 && *height >= 160)
                    .min_by_key(|(_, _, _, area)| *area)
                    .or_else(|| candidates.iter().max_by_key(|(_, _, _, area)| *area))
                    .map(|(file, width, height, _)| (*file, *width, *height))
            });
            let media = chosen.and_then(|(file, width, height)| {
                media_file(Some(file), "Photo", "image/jpeg").map(|mut media| {
                    media.width = width;
                    media.height = height;
                    media
                })
            });
            (
                if caption.is_empty() {
                    "Photo".into()
                } else {
                    caption
                },
                media.map(MessageKind::Photo).unwrap_or_else(|| {
                    MessageKind::Unsupported("Photo (file metadata unavailable)".into())
                }),
            )
        }
        "messageDocument" => {
            let document = &content["document"];
            let file = document.get("document");
            let name = document
                .get("file_name")
                .and_then(Value::as_str)
                .unwrap_or("Document")
                .to_owned();
            let mime = document
                .get("mime_type")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let media = media_file(file, &name, &mime);
            (
                if caption.is_empty() {
                    name.clone()
                } else {
                    caption
                },
                media
                    .map(MessageKind::Document)
                    .unwrap_or_else(|| MessageKind::Unsupported(format!("Document: {name}"))),
            )
        }
        "messageVideo" => {
            let video = &content["video"];
            let name = video
                .get("file_name")
                .and_then(Value::as_str)
                .unwrap_or("Video")
                .to_owned();
            let mime = video
                .get("mime_type")
                .and_then(Value::as_str)
                .unwrap_or("video/mp4")
                .to_owned();
            let mut media = media_file(video.get("video"), &name, &mime);
            if let Some(media) = media.as_mut() {
                media.duration = video.get("duration").and_then(Value::as_i64).unwrap_or(0) as i32;
            }
            (
                if caption.is_empty() {
                    name.clone()
                } else {
                    caption
                },
                media
                    .map(MessageKind::Video)
                    .unwrap_or_else(|| MessageKind::Unsupported(format!("Video: {name}"))),
            )
        }
        "messageSticker" => {
            let sticker = &content["sticker"];
            let name = sticker
                .get("emoji")
                .and_then(Value::as_str)
                .unwrap_or("Sticker")
                .to_owned();
            (
                name.clone(),
                media_file(sticker.get("sticker"), &name, "image/webp")
                    .map(MessageKind::Sticker)
                    .unwrap_or_else(|| MessageKind::Unsupported("Sticker".into())),
            )
        }
        "messageVoiceNote" => {
            let voice = &content["voice_note"];
            let duration = voice.get("duration").and_then(Value::as_i64).unwrap_or(0) as i32;
            let mut media = media_file(voice.get("voice"), "Voice message", "audio/ogg");
            if let Some(media) = media.as_mut() {
                media.duration = duration;
            }
            (
                format!("Voice message · {}s", duration),
                media
                    .map(MessageKind::Voice)
                    .unwrap_or_else(|| MessageKind::Unsupported("Voice message".into())),
            )
        }
        "messageAudio" => {
            let audio = &content["audio"];
            let name = audio
                .get("title")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .or_else(|| audio.get("file_name").and_then(Value::as_str))
                .unwrap_or("Audio")
                .to_owned();
            let media = media_file(
                audio.get("audio"),
                &name,
                audio
                    .get("mime_type")
                    .and_then(Value::as_str)
                    .unwrap_or("audio/mpeg"),
            );
            (
                if caption.is_empty() {
                    name.clone()
                } else {
                    caption
                },
                media
                    .map(MessageKind::Audio)
                    .unwrap_or_else(|| MessageKind::Unsupported(format!("Audio: {name}"))),
            )
        }
        "messageAnimation" => {
            let animation = &content["animation"];
            let name = animation
                .get("file_name")
                .and_then(Value::as_str)
                .unwrap_or("Animation")
                .to_owned();
            let media = media_file(animation.get("animation"), &name, "video/mp4");
            (
                if caption.is_empty() {
                    name.clone()
                } else {
                    caption
                },
                media
                    .map(MessageKind::Video)
                    .unwrap_or_else(|| MessageKind::Unsupported(format!("Animation: {name}"))),
            )
        }
        "messageAnimatedEmoji" => (
            content
                .pointer("/emoji/emoji")
                .and_then(Value::as_str)
                .unwrap_or("Animated emoji")
                .to_owned(),
            MessageKind::Unsupported("Animated emoji (preview unavailable)".into()),
        ),
        "messageContact" => (
            "Contact card".into(),
            MessageKind::Unsupported("Contact card".into()),
        ),
        "messageLocation" | "messageVenue" => (
            "Location".into(),
            MessageKind::Unsupported("Location card".into()),
        ),
        other
            if other.starts_with("messageChat")
                || other.starts_with("messagePin")
                || other.starts_with("messagePayment") =>
        {
            let label = service_label(other);
            (label.clone(), MessageKind::Service(label))
        }
        other => {
            let label = format!("Unsupported message ({other})");
            (label.clone(), MessageKind::Unsupported(label))
        }
    }
}

fn media_file(file: Option<&Value>, name: &str, mime_type: &str) -> Option<MediaFile> {
    let file = file?;
    let id = file.get("id")?.as_i64()? as i32;
    let local = &file["local"];
    let downloaded = local
        .get("is_downloading_completed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let local_path = local
        .get("path")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .map(str::to_owned);
    Some(MediaFile {
        id,
        name: name.to_owned(),
        mime_type: if mime_type.is_empty() {
            "application/octet-stream".into()
        } else {
            mime_type.to_owned()
        },
        size: file
            .get("size")
            .and_then(Value::as_i64)
            .or_else(|| file.get("expected_size").and_then(Value::as_i64))
            .unwrap_or(0),
        width: 0,
        height: 0,
        duration: 0,
        local_path,
        downloaded,
        downloading: local
            .get("is_downloading_active")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        downloaded_size: local
            .get("downloaded_prefix_size")
            .and_then(Value::as_i64)
            .or_else(|| local.get("downloaded_size").and_then(Value::as_i64))
            .unwrap_or(0),
    })
}

fn service_label(kind: &str) -> String {
    match kind {
        "messageChatAddMembers" => "Members added".into(),
        "messageChatDeleteMember" => "Member removed".into(),
        "messageChatJoinByLink" => "Joined via invite link".into(),
        "messageChatJoinByRequest" => "Join request approved".into(),
        "messageChatSetTitle" => "Chat title changed".into(),
        "messageChatChangePhoto" => "Chat photo changed".into(),
        "messageChatDeletePhoto" => "Chat photo removed".into(),
        "messagePinMessage" => "Message pinned".into(),
        "messagePaymentSuccessful" => "Payment completed".into(),
        _ => "Service message".into(),
    }
}

fn data_path() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share")
        })
        .join("telerust")
}

/// Read GNOME's manual proxy configuration without logging host credentials.
/// The proxy is passed only to TDLib in memory and is never written to app config.
fn gnome_proxy_request() -> Option<Value> {
    use gio::prelude::*;

    if let Some(source) = gio::SettingsSchemaSource::default() {
        if let Some(root_schema) = source.lookup("org.gnome.system.proxy", true) {
            let root = gio::Settings::new_full(&root_schema, None::<&gio::SettingsBackend>, None);
            if root.string("mode").as_str() == "manual" {
                for (scheme, proxy_type) in [
                    ("https", "proxyTypeHttp"),
                    ("http", "proxyTypeHttp"),
                    ("socks", "proxyTypeSocks5"),
                ] {
                    let schema_id = format!("org.gnome.system.proxy.{scheme}");
                    let Some(schema) = source.lookup(&schema_id, true) else {
                        continue;
                    };
                    let settings =
                        gio::Settings::new_full(&schema, None::<&gio::SettingsBackend>, None);
                    let host = settings.string("host");
                    let port = settings.int("port");
                    if host.is_empty() || !(1..=u16::MAX as i32).contains(&port) {
                        continue;
                    }
                    let (username, password) = if schema.has_key("use-authentication")
                        && settings.boolean("use-authentication")
                    {
                        (
                            settings.string("authentication-user").to_string(),
                            settings.string("authentication-password").to_string(),
                        )
                    } else {
                        (String::new(), String::new())
                    };
                    return Some(proxy_request(
                        host.as_str(),
                        port as u16,
                        proxy_type,
                        &username,
                        &password,
                    ));
                }
            }
        }
    }

    [
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ]
    .iter()
    .filter_map(|key| std::env::var(key).ok())
    .find_map(|value| proxy_request_from_url(&value))
}

fn proxy_request(host: &str, port: u16, proxy_type: &str, username: &str, password: &str) -> Value {
    let kind = if proxy_type == "proxyTypeHttp" {
        json!({
            "@type": proxy_type,
            "username": username,
            "password": password,
            "http_only": false
        })
    } else {
        json!({
            "@type": proxy_type,
            "username": username,
            "password": password
        })
    };
    json!({
        "@type":"addProxy",
        "enable":true,
        "proxy":{
            "@type":"proxy",
            "server":host,
            "port":port,
            "type":kind
        }
    })
}

fn proxy_request_from_url(value: &str) -> Option<Value> {
    let (scheme, rest) = value.trim().split_once("://")?;
    let proxy_type = match scheme.to_ascii_lowercase().as_str() {
        "http" | "https" => "proxyTypeHttp",
        "socks5" | "socks5h" => "proxyTypeSocks5",
        _ => return None,
    };
    let authority = rest.split(['/', '?', '#']).next()?;
    let (userinfo, host_port) = authority
        .rsplit_once('@')
        .map_or((None, authority), |(userinfo, host)| (Some(userinfo), host));
    let (host, port) = if let Some(bracketed) = host_port.strip_prefix('[') {
        let (host, tail) = bracketed.split_once(']')?;
        let port = tail.strip_prefix(':')?.parse::<u16>().ok()?;
        (host, port)
    } else {
        let (host, port) = host_port.rsplit_once(':')?;
        if host.contains(':') {
            return None;
        }
        (host, port.parse::<u16>().ok()?)
    };
    if host.is_empty() || port == 0 {
        return None;
    }
    let (username, password) = if let Some(userinfo) = userinfo {
        let (username, password) = userinfo.split_once(':').unwrap_or((userinfo, ""));
        (percent_decode(username)?, percent_decode(password)?)
    } else {
        (String::new(), String::new())
    };
    Some(proxy_request(host, port, proxy_type, &username, &password))
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            decoded.push((hex_value(high)? << 4) | hex_value(low)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn get_database_key(api_id: i32, environment: &str) -> Result<String, BackendError> {
    let account = format!("tdlib-db-key-{api_id}-{environment}");
    let entry =
        keyring::Entry::new("org.telerust.Session", &account).map_err(|_| BackendError::Keyring)?;
    match entry.get_password() {
        Ok(key) => Ok(key),
        Err(keyring::Error::NoEntry) => {
            let mut bytes = [0u8; 32];
            getrandom::fill(&mut bytes).map_err(|_| BackendError::Keyring)?;
            let key: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
            entry
                .set_password(&key)
                .map_err(|_| BackendError::Keyring)?;
            Ok(key)
        }
        Err(_) => Err(BackendError::Keyring),
    }
}

type CreateClientIdFn = unsafe extern "C" fn() -> i32;
type SendFn = unsafe extern "C" fn(i32, *const c_char);
type ReceiveFn = unsafe extern "C" fn(f64) -> *const c_char;
type ExecuteFn = unsafe extern "C" fn(*const c_char) -> *const c_char;

struct TdClient {
    _library: Library,
    client_id: i32,
    send_fn: SendFn,
    receive_fn: ReceiveFn,
}

impl TdClient {
    fn load(explicit: Option<&Path>) -> Result<Self, BackendError> {
        let mut candidates = Vec::new();
        if let Some(path) = explicit {
            candidates.push(path.to_owned());
        }
        candidates.push(data_path().join("lib/libtdjson.so"));
        candidates.extend(
            [
                "libtdjson.so",
                "libtdjson-x64-glibc.so",
                "/usr/lib64/libtdjson.so",
                "/usr/lib64/libtdjson-x64-glibc.so",
                "/usr/lib/libtdjson.so",
                "/usr/local/lib/libtdjson.so",
                "/usr/local/lib64/libtdjson.so",
            ]
            .into_iter()
            .map(PathBuf::from),
        );
        candidates.dedup();
        let found_file = candidates.iter().any(|candidate| candidate.is_file());
        let mut loaded = None;
        for candidate in candidates {
            // SAFETY: The library is retained in Self for the entire lifetime of
            // every function pointer and TDLib client created from it.
            if let Ok(library) = unsafe { Library::new(candidate) } {
                loaded = Some(library);
                break;
            }
        }
        let library = loaded.ok_or(if found_file {
            BackendError::UnloadableLibrary
        } else {
            BackendError::MissingLibrary
        })?;
        // SAFETY: These symbol names and signatures match TDLib's documented
        // modern JSON C interface. Function pointers stay within library lifetime.
        let (create_fn, send_fn, receive_fn, execute_fn) = unsafe {
            (
                *library
                    .get::<CreateClientIdFn>(b"td_create_client_id\0")
                    .map_err(|_| BackendError::MissingSymbol)?,
                *library
                    .get::<SendFn>(b"td_send\0")
                    .map_err(|_| BackendError::MissingSymbol)?,
                *library
                    .get::<ReceiveFn>(b"td_receive\0")
                    .map_err(|_| BackendError::MissingSymbol)?,
                *library
                    .get::<ExecuteFn>(b"td_execute\0")
                    .map_err(|_| BackendError::MissingSymbol)?,
            )
        };
        let quiet_request =
            CString::new(r#"{"@type":"setLogVerbosityLevel","new_verbosity_level":0}"#)
                .expect("static JSON has no NUL");
        // SAFETY: `td_execute` accepts a JSON request independent of a client
        // ID; setting log verbosity is documented as synchronous.
        unsafe {
            execute_fn(quiet_request.as_ptr());
        }
        let discard_request =
            CString::new(r#"{"@type":"setLogStream","log_stream":{"@type":"logStreamEmpty"}}"#)
                .expect("static JSON has no NUL");
        // SAFETY: setLogStream is synchronous and independent of client ID;
        // configure an empty stream before creating the client instance.
        unsafe {
            execute_fn(discard_request.as_ptr());
        }
        // SAFETY: `td_create_client_id` is documented as a zero-argument C function.
        let client_id = unsafe { create_fn() };
        Ok(Self {
            _library: library,
            client_id,
            send_fn,
            receive_fn,
        })
    }

    fn send(&mut self, request: &Value) {
        if let Ok(request) = CString::new(request.to_string()) {
            // SAFETY: The pointer is a valid NUL-terminated JSON request and
            // the client ID was created by the same TDLib library.
            unsafe {
                (self.send_fn)(self.client_id, request.as_ptr());
            }
        }
    }

    fn receive(&mut self, timeout_seconds: f64) -> Option<Value> {
        // SAFETY: This is the process's only `td_receive` caller, and the
        // timeout is finite. This application owns only one TDLib client ID.
        let pointer = unsafe { (self.receive_fn)(timeout_seconds) };
        if pointer.is_null() {
            return None;
        }
        // SAFETY: TDLib guarantees a NUL-terminated UTF-8 buffer valid until
        // the next receive/execute call; copy it before another receive.
        let bytes = unsafe { CStr::from_ptr(pointer) }.to_bytes().to_vec();
        serde_json::from_slice(&bytes).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn history_requests_use_tdlib_page_limits_and_request_correlation() {
        let initial = history_query(42, 0, 17, PageDirection::Initial);
        assert_eq!(initial["@type"], "getChatHistory");
        assert_eq!(initial["chat_id"], 42);
        assert_eq!(initial["from_message_id"], 0);
        assert_eq!(initial["limit"], PAGE_SIZE);
        assert_eq!(initial["@extra"], 17);
        let older = history_query(42, 123, 18, PageDirection::Older);
        assert_eq!(older["from_message_id"], 123);
        assert_eq!(older["@extra"], 18);
        let newer = history_query(42, 200, 19, PageDirection::Newer);
        assert_eq!(newer["offset"], 1 - PAGE_SIZE);
    }

    #[test]
    fn tdlib_authorization_states_map_to_the_expected_ui_transitions() {
        assert_eq!(
            authorization_state_from_tdlib("authorizationStateWaitOtherDeviceConfirmation"),
            AuthorizationState::WaitingForQrConfirmation
        );
        assert_eq!(
            authorization_state_from_tdlib("authorizationStateWaitCode"),
            AuthorizationState::WaitingForCode
        );
        assert_eq!(
            authorization_state_from_tdlib("authorizationStateWaitPassword"),
            AuthorizationState::WaitingForPassword
        );
        assert_eq!(
            authorization_state_from_tdlib("authorizationStateReady"),
            AuthorizationState::Ready
        );
        assert_eq!(
            authorization_state_from_tdlib("authorizationStateLoggingOut"),
            AuthorizationState::LoggingOut
        );
        assert_eq!(
            authorization_state_from_tdlib("authorizationStateClosed"),
            AuthorizationState::Closed
        );
    }

    #[test]
    fn parses_text_message_without_logging_or_network() {
        let value = json!({"id":11,"is_outgoing":true,"content":{"@type":"messageText","text":{"text":"hello"}}});
        assert_eq!(
            message_view(&value),
            MessageView {
                id: 11,
                date: 0,
                sender_id: String::new(),
                text: "hello".into(),
                kind: MessageKind::Text,
                outgoing: true,
                delivery: DeliveryState::Sent,
            }
        );
    }

    #[test]
    fn message_preview_handles_non_text_content() {
        assert_eq!(
            message_text(&json!({"content":{"@type":"messagePhoto"}})),
            "Photo"
        );
    }

    #[test]
    fn empty_and_unknown_message_contents_are_informational() {
        let empty =
            message_view(&json!({"id":5,"content":{"@type":"messageText","text":{"text":"  "}}}));
        assert_eq!(empty.text, "Empty text message");
        assert!(matches!(empty.kind, MessageKind::Unsupported(_)));
        let unknown = message_view(&json!({"id":6,"content":{"@type":"messagePoll"}}));
        assert!(unknown.text.contains("messagePoll"));
        assert!(matches!(unknown.kind, MessageKind::Unsupported(_)));
    }

    #[test]
    fn document_content_preserves_filename_mime_size_and_tdlib_file_id() {
        let view = message_view(
            &json!({"id":7,"content":{"@type":"messageDocument","caption":{"text":"notes"},"document":{"file_name":"notes.pdf","mime_type":"application/pdf","document":{"id":73,"size":4096,"local":{"path":"","is_downloading_completed":false}}}}}),
        );
        assert_eq!(view.text, "notes");
        let MessageKind::Document(file) = view.kind else {
            panic!("expected document model");
        };
        assert_eq!(file.id, 73);
        assert_eq!(file.name, "notes.pdf");
        assert_eq!(file.mime_type, "application/pdf");
        assert_eq!(file.size, 4096);
        assert!(!file.downloaded);
    }

    #[test]
    fn photo_parser_selects_small_renderable_variant_with_dimensions() {
        let view = message_view(
            &json!({"id":8,"content":{"@type":"messagePhoto","photo":{"sizes":[{"width":90,"height":90,"photo":{"id":10,"size":1000,"local":{"path":"","is_downloading_completed":false}}},{"width":320,"height":240,"photo":{"id":11,"size":12000,"local":{"path":"","is_downloading_completed":false}}},{"width":1280,"height":960,"photo":{"id":12,"size":90000,"local":{"path":"","is_downloading_completed":false}}}]},"caption":{"text":"caption"}}}),
        );
        assert_eq!(view.text, "caption");
        let MessageKind::Photo(file) = view.kind else {
            panic!("expected photo model");
        };
        assert_eq!((file.id, file.width, file.height), (11, 320, 240));
    }

    #[test]
    fn chat_photo_file_is_carried_for_lazy_avatar_loading() {
        let mut chats = HashMap::new();
        upsert_chat(
            &json!({"id":14,"title":"Synthetic group","photo":{"small":{"id":140,"local":{"path":"","is_downloading_completed":false}}},"type":{"@type":"chatTypeBasicGroup"}}),
            &mut chats,
        );
        let chat = chats.get(&14).unwrap();
        assert_eq!(chat.photo_file_id, Some(140));
        assert_eq!(chat.photo_path, None);
    }

    #[test]
    fn avatar_photo_update_replaces_file_and_can_clear_photo() {
        let mut chat = ChatSummary {
            id: 14,
            title: "Synthetic group".into(),
            preview: String::new(),
            sort_date: 0,
            is_channel: false,
            photo_file_id: Some(140),
            photo_path: None,
        };
        update_chat_photo(
            &mut chat,
            &json!({"small":{"id":141,"local":{"path":"/synthetic/new-avatar.jpg"}}}),
        );
        assert_eq!(chat.photo_file_id, Some(141));
        assert_eq!(
            chat.photo_path.as_deref(),
            Some("/synthetic/new-avatar.jpg")
        );
        update_chat_photo(&mut chat, &Value::Null);
        assert_eq!(chat.photo_file_id, None);
        assert_eq!(chat.photo_path, None);
    }

    #[test]
    fn media_download_request_is_correlated_and_nonblocking() {
        let request = download_file_query(55);
        assert_eq!(request["@type"], "downloadFile");
        assert_eq!(request["file_id"], 55);
        assert_eq!(request["synchronous"], false);
        assert_eq!(request["@extra"], "telerust-file-55");
    }

    #[test]
    fn channel_chat_updates_are_not_exposed_until_sponsored_support_exists() {
        let (sender, receiver) = async_channel::bounded(1);
        let chats = HashMap::from([(
            9,
            ChatSummary {
                id: 9,
                title: "Channel".into(),
                preview: "hidden channel preview".into(),
                sort_date: 12,
                is_channel: true,
                photo_file_id: None,
                photo_path: None,
            },
        )]);
        emit_chats(&chats, &sender);
        let event = receiver.try_recv().unwrap();
        assert!(matches!(event, BackendEvent::Chats(items) if items.is_empty()));
    }

    #[test]
    fn invalid_api_credentials_get_actionable_redacted_diagnostic() {
        let error = tdlib_error_message(400, "API_ID_INVALID");
        assert!(error.contains("saved API credentials"));
        assert!(error.contains("values are hidden"));
    }

    #[test]
    fn unknown_tdlib_errors_do_not_echo_server_text() {
        let error = tdlib_error_message(500, "private server detail");
        assert_eq!(
            error,
            "Telegram request failed (code 500). Check the connection and retry."
        );
    }

    #[test]
    fn proxy_url_becomes_tdlib_request_without_exposing_credentials() {
        let request = proxy_request_from_url("http://demo%40user:p%3Aword@127.0.0.1:1080/")
            .expect("synthetic proxy URL is valid");
        assert_eq!(request["@type"], "addProxy");
        assert_eq!(request["proxy"]["server"], "127.0.0.1");
        assert_eq!(request["proxy"]["port"], 1080);
        assert_eq!(request["proxy"]["type"]["username"], "demo@user");
        assert_eq!(request["proxy"]["type"]["password"], "p:word");
    }

    #[test]
    fn proxy_url_parser_rejects_invalid_ports_and_schemes() {
        assert!(proxy_request_from_url("http://127.0.0.1:0").is_none());
        assert!(proxy_request_from_url("ftp://127.0.0.1:21").is_none());
    }

    #[test]
    fn connection_state_updates_have_actionable_network_status() {
        assert!(connection_status("connectionStateWaitingForNetwork")
            .unwrap()
            .contains("GNOME network and proxy"));
        assert_eq!(
            connection_status("connectionStateReady"),
            Some("Connected to Telegram.")
        );
        assert_eq!(connection_status("unknownConnectionState"), None);
    }
}
