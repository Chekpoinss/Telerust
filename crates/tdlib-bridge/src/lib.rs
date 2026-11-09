//! Asynchronous TDLib adapter using its current JSON C ABI.
//! All unsafe code and native pointers are contained in this crate.

use async_channel::{Receiver, Sender};
use libloading::Library;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::c_char,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use telerust_client_core::{events::AuthorizationState, settings::Settings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSummary {
    pub id: i64,
    pub title: String,
    pub preview: String,
    pub sort_date: i64,
    pub is_channel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageView {
    pub id: i64,
    pub text: String,
    pub outgoing: bool,
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
        messages: Vec<MessageView>,
    },
    NewMessage {
        chat_id: i64,
        message: MessageView,
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
    OpenChat(i64),
    SendText { chat_id: i64, text: String },
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
    MissingSymbol,
    Keyring,
    Storage,
}

impl BackendError {
    fn as_str(&self) -> &'static str {
        match self {
            Self::MissingLibrary => "TDLib is not installed. Install libtdjson and restart.",
            Self::MissingSymbol => {
                "The TDLib library does not provide the required JSON C interface."
            }
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
    let key = get_database_key(settings.api_id, environment)?;
    let mut client = TdClient::load(settings.tdlib_library.as_deref())?;
    client.send(&json!({"@type":"getAuthorizationState"}));

    let mut chats: HashMap<i64, ChatSummary> = HashMap::new();
    let mut selected_chat = None;
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
                BackendCommand::OpenChat(chat_id) => {
                    if chats.get(&chat_id).is_some_and(|chat| chat.is_channel) {
                        let _ = events.send_blocking(BackendEvent::Error(
                            "Channel chats are disabled until sponsored message support is implemented.".into(),
                        ));
                        continue;
                    }
                    selected_chat = Some(chat_id);
                    client.send(&json!({
                        "@type":"getChatHistory", "chat_id":chat_id,
                        "from_message_id":0, "offset":0, "limit":50, "only_local":false
                    }));
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
        events,
        closed,
        prefer_qr,
    } = context;
    match update.get("@type").and_then(Value::as_str).unwrap_or("") {
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
                    client
                        .send(&json!({"@type":"checkDatabaseEncryptionKey","encryption_key":key}));
                    (AuthorizationState::WaitingForParameters, None)
                }
                "authorizationStateWaitPhoneNumber" => {
                    if *prefer_qr {
                        client.send(
                            &json!({"@type":"requestQrCodeAuthentication","other_user_ids":[]}),
                        );
                        return;
                    }
                    (
                        AuthorizationState::WaitingForPhoneNumber,
                        Some(AuthPrompt::PhoneNumber),
                    )
                }
                "authorizationStateWaitCode" => {
                    (AuthorizationState::WaitingForCode, Some(AuthPrompt::Code))
                }
                "authorizationStateWaitPassword" => (
                    AuthorizationState::WaitingForPassword,
                    Some(AuthPrompt::Password),
                ),
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
        "updateChatTitle" | "updateChatLastMessage" | "updateChatPosition" => {
            let id = update.get("chat_id").and_then(Value::as_i64);
            if let Some(id) = id {
                let chat = chats.entry(id).or_insert_with(|| ChatSummary {
                    id,
                    title: format!("Chat {id}"),
                    preview: String::new(),
                    sort_date: 0,
                    is_channel: false,
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
        "messages" => {
            if let Some(messages) = update.get("messages").and_then(Value::as_array) {
                let chat_id = messages
                    .first()
                    .and_then(|m| m.get("chat_id"))
                    .and_then(Value::as_i64)
                    .or(*selected_chat)
                    .unwrap_or_default();
                let messages = messages.iter().map(message_view).collect();
                let _ = events.send_blocking(BackendEvent::History { chat_id, messages });
            }
        }
        "error" => {
            let code = update.get("code").and_then(Value::as_i64).unwrap_or(0);
            let _ = events.send_blocking(BackendEvent::Error(format!(
                "Telegram request failed (code {code})."
            )));
        }
        "updateMessageSendFailed" => {
            let code = update
                .pointer("/error_code")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let _ = events.send_blocking(BackendEvent::Error(format!(
                "Message was not sent (code {code})."
            )));
        }
        _ => {}
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
    });
    item.title = title;
    item.is_channel = is_channel;
    if let Some(last) = last {
        item.preview = message_text(last);
        item.sort_date = last.get("date").and_then(Value::as_i64).unwrap_or(0);
    }
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
    MessageView {
        id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
        text: message_text(value),
        outgoing: value
            .get("is_outgoing")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

fn message_text(value: &Value) -> String {
    let content = &value["content"];
    match content.get("@type").and_then(Value::as_str).unwrap_or("") {
        "messageText" => content
            .pointer("/text/text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        "messagePhoto" => "Photo".into(),
        "messageVideo" => "Video".into(),
        "messageDocument" => "File".into(),
        "messageSticker" => "Sticker".into(),
        "messageVoiceNote" => "Voice message".into(),
        "messageAnimatedEmoji" => "Emoji".into(),
        _ => "Message".into(),
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
        let candidates = if let Some(path) = explicit {
            vec![path.to_owned()]
        } else {
            [
                "libtdjson.so",
                "libtdjson-x64-glibc.so",
                "/usr/lib64/libtdjson.so",
                "/usr/lib64/libtdjson-x64-glibc.so",
                "/usr/local/lib/libtdjson.so",
                "/usr/local/lib64/libtdjson.so",
            ]
            .into_iter()
            .map(PathBuf::from)
            .collect()
        };
        let mut loaded = None;
        for candidate in candidates {
            // SAFETY: The library is retained in Self for the entire lifetime of
            // every function pointer and TDLib client created from it.
            if let Ok(library) = unsafe { Library::new(candidate) } {
                loaded = Some(library);
                break;
            }
        }
        let library = loaded.ok_or(BackendError::MissingLibrary)?;
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
                text: "hello".into(),
                outgoing: true
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
            },
        )]);
        emit_chats(&chats, &sender);
        let event = receiver.try_recv().unwrap();
        assert!(matches!(event, BackendEvent::Chats(items) if items.is_empty()));
    }
}
