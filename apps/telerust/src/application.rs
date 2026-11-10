use crate::{
    services::telegram,
    ui::{
        chat_header::ChatHeader, chat_list::ChatList, composer::Composer, message_list::MessageList,
    },
};
use adw::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use telerust_client_core::{
    events::AuthorizationState,
    history::{
        ChatId, DeliveryState, HistoryPage, HistoryWindow, Message, MessageKind, PageDirection,
    },
    settings::Settings,
};
use telerust_tdlib_bridge::{AuthPrompt, BackendCommand, BackendEvent, ChatSummary, MessageView};

#[derive(Clone)]
struct Ui {
    stack: gtk::Stack,
    status: gtk::Label,
    auth_title: gtk::Label,
    auth_input: gtk::Entry,
    auth_password: gtk::Entry,
    auth_button: gtk::Button,
    qr_area: gtk::DrawingArea,
    use_phone_button: gtk::Button,
    use_qr_button: gtk::Button,
    credentials_button: gtk::Button,
    logout_button: gtk::Button,
    current_prompt: Rc<RefCell<AuthPrompt>>,
    chats: ChatList,
    chat_header: ChatHeader,
    messages: MessageList,
    composer: Composer,
    selected_chat: Rc<Cell<Option<i64>>>,
    history: Rc<RefCell<HistoryWindow>>,
    command_tx: async_channel::Sender<BackendCommand>,
}

pub fn run() {
    let synthetic_count = std::env::var("TELERUST_SYNTHETIC_HISTORY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|count| *count > 0);
    let application = adw::Application::builder()
        .application_id(if synthetic_count.is_some() {
            "org.telerust.Client.Synthetic"
        } else {
            "org.telerust.Client"
        })
        .build();
    application.connect_activate(move |app| build_window(app, synthetic_count));
    application.run();
}

fn build_window(app: &adw::Application, synthetic_count: Option<usize>) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Unofficial Telerust")
        .default_width(1020)
        .default_height(700)
        .build();
    window.add_css_class("telerust-window");

    let (command_tx, event_rx) = if synthetic_count.is_some() {
        let (command_tx, command_rx) = async_channel::bounded(1);
        let (event_tx, event_rx) = async_channel::bounded(1);
        drop(command_rx);
        drop(event_tx);
        (command_tx, event_rx)
    } else {
        let backend = telegram::start();
        (backend.commands, backend.events)
    };
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let status = gtk::Label::new(None);
    status.set_xalign(0.0);
    status.set_margin_start(12);
    status.set_margin_end(12);
    status.set_margin_top(6);
    status.set_margin_bottom(6);
    status.add_css_class("dim-label");
    status.set_wrap(true);
    status.set_max_width_chars(120);
    content.append(&status);

    let stack = gtk::Stack::builder()
        .hexpand(true)
        .vexpand(true)
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    content.append(&stack);

    let (
        login_page,
        auth_title,
        auth_input,
        auth_password,
        auth_button,
        qr_area,
        use_phone_button,
        use_qr_button,
        credentials_button,
    ) = login_view();
    stack.add_named(&login_page, Some("login"));
    let main_page = main_view(command_tx.clone());
    stack.add_named(&main_page.0, Some("main"));
    stack.set_visible_child_name("login");
    window.set_content(Some(&content));
    crate::ui::theme::install();

    let ui = Ui {
        stack,
        status,
        auth_title,
        auth_input,
        auth_password,
        auth_button,
        qr_area,
        use_phone_button,
        use_qr_button,
        credentials_button,
        logout_button: main_page.5,
        current_prompt: Rc::new(RefCell::new(AuthPrompt::PhoneNumber)),
        chats: main_page.1,
        chat_header: main_page.2,
        messages: main_page.3,
        composer: main_page.4,
        selected_chat: Rc::new(Cell::new(None)),
        history: Rc::new(RefCell::new(HistoryWindow::default())),
        command_tx,
    };

    wire_auth(&ui);
    wire_credentials(&ui, &window);
    wire_messages(&ui);
    wire_logout(&ui);
    if let Some(count) = synthetic_count {
        show_synthetic_history(&ui, count.min(10_000));
    } else {
        attach_events(&ui, event_rx);
    }

    let command_tx = ui.command_tx.clone();
    app.connect_shutdown(move |_| {
        let _ = command_tx.try_send(BackendCommand::Shutdown);
    });
    window.present();
}

fn login_view() -> (
    gtk::Box,
    gtk::Label,
    gtk::Entry,
    gtk::Entry,
    gtk::Button,
    gtk::DrawingArea,
    gtk::Button,
    gtk::Button,
    gtk::Button,
) {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_halign(gtk::Align::Center);
    outer.set_valign(gtk::Align::Center);
    outer.set_margin_top(32);
    outer.set_margin_bottom(32);
    outer.set_margin_start(32);
    outer.set_margin_end(32);
    outer.set_width_request(400);

    let title = gtk::Label::new(Some("Unofficial Telerust"));
    title.add_css_class("title-1");
    outer.append(&title);
    let disclosure = gtk::Label::new(Some("An unofficial Telegram client. Use your own API credentials from my.telegram.org/apps. Production servers are used by default."));
    disclosure.set_wrap(true);
    disclosure.set_margin_top(8);
    disclosure.set_margin_bottom(20);
    outer.append(&disclosure);

    let qr_help = gtk::Label::new(Some(
        "Scan with Telegram on your phone: Settings → Devices → Link Desktop Device.",
    ));
    qr_help.set_wrap(true);
    qr_help.set_justify(gtk::Justification::Center);
    outer.append(&qr_help);
    let qr_area = gtk::DrawingArea::new();
    qr_area.set_content_width(280);
    qr_area.set_content_height(280);
    qr_area.set_size_request(280, 280);
    qr_area.set_halign(gtk::Align::Center);
    qr_area.set_margin_top(12);
    qr_area.set_margin_bottom(12);
    outer.append(&qr_area);
    qr_area.set_visible(false);

    let prompt = gtk::Label::new(Some("Loading TDLib authorization state…"));
    prompt.set_xalign(0.0);
    outer.append(&prompt);
    let input = gtk::Entry::new();
    input.set_placeholder_text(Some("Phone number"));
    input.set_margin_top(8);
    outer.append(&input);
    input.set_visible(false);
    let password = gtk::Entry::new();
    password.set_placeholder_text(Some("Two-step verification password"));
    password.set_visibility(false);
    password.set_margin_top(8);
    password.set_visible(false);
    outer.append(&password);
    let button = gtk::Button::with_label("Continue");
    button.add_css_class("suggested-action");
    button.set_margin_top(12);
    outer.append(&button);
    button.set_visible(false);
    let use_phone = gtk::Button::with_label("Use phone number instead");
    outer.append(&use_phone);
    let use_qr = gtk::Button::with_label("Use QR code instead");
    use_qr.set_visible(false);
    outer.append(&use_qr);
    let credentials = gtk::Button::with_label("Set up Telegram API credentials");
    credentials.set_margin_top(12);
    outer.append(&credentials);
    (
        outer,
        prompt,
        input,
        password,
        button,
        qr_area,
        use_phone,
        use_qr,
        credentials,
    )
}

fn wire_credentials(ui: &Ui, window: &adw::ApplicationWindow) {
    let button = ui.credentials_button.clone();
    let ui = ui.clone();
    let parent = window.clone();
    button.connect_clicked(move |_| {
        let dialog = gtk::Window::builder()
            .title("Telegram API credentials")
            .transient_for(&parent)
            .modal(true)
            .default_width(420)
            .build();
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 8);
        layout.set_margin_top(12);
        layout.set_margin_bottom(12);
        layout.set_margin_start(12);
        layout.set_margin_end(12);
        let help = gtk::Label::new(Some("Create an application at https://my.telegram.org/apps, then enter its api_id and api_hash here. They are saved locally with owner-only permissions and are never logged."));
        help.set_wrap(true);
        layout.append(&help);
        let api_id = gtk::Entry::new();
        api_id.set_placeholder_text(Some("api_id"));
        api_id.set_input_purpose(gtk::InputPurpose::Digits);
        layout.append(&api_id);
        let api_hash = gtk::Entry::new();
        api_hash.set_placeholder_text(Some("api_hash"));
        api_hash.set_visibility(false);
        layout.append(&api_hash);
        let test_dc = gtk::CheckButton::with_label("Use Telegram test DC (requires a separate test account)");
        layout.append(&test_dc);
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        actions.set_halign(gtk::Align::End);
        let cancel = gtk::Button::with_label("Cancel");
        let save = gtk::Button::with_label("Save");
        save.add_css_class("suggested-action");
        actions.append(&cancel);
        actions.append(&save);
        layout.append(&actions);
        dialog.set_child(Some(&layout));
        let dialog_cancel = dialog.clone();
        cancel.connect_clicked(move |_| dialog_cancel.close());
        let ui = ui.clone();
        let dialog_save = dialog.clone();
        save.connect_clicked(move |_| {
            let settings = api_id.text().parse::<i32>().ok().map(|api_id| Settings {
                api_id,
                api_hash: api_hash.text().to_string(),
                use_test_dc: test_dc.is_active(),
                tdlib_library: None,
            });
            match settings.and_then(|settings| settings.save().ok()) {
                Some(()) => ui.status.set_text("Credentials saved. Restart Telerust to connect."),
                None => ui.status.set_text("Credentials were not saved. Check api_id and api_hash; values are not displayed."),
            }
            dialog_save.close();
        });
        dialog.present();
    });
}

// Return page plus widgets which need event updates.
fn main_view(
    command_tx: async_channel::Sender<BackendCommand>,
) -> (
    gtk::Box,
    ChatList,
    ChatHeader,
    MessageList,
    Composer,
    gtk::Button,
) {
    let split = gtk::Paned::new(gtk::Orientation::Horizontal);
    split.set_position(320);
    split.set_wide_handle(true);

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let heading_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    heading_row.set_margin_start(12);
    heading_row.set_margin_end(12);
    heading_row.set_margin_top(12);
    heading_row.set_margin_bottom(8);
    let heading = gtk::Label::new(Some("Chats"));
    heading.add_css_class("title-2");
    heading.set_xalign(0.0);
    heading.set_hexpand(true);
    let logout = gtk::Button::with_label("Log out");
    logout.add_css_class("flat");
    heading_row.append(&heading);
    heading_row.append(&logout);
    sidebar.append(&heading_row);
    let chats = ChatList::new(command_tx.clone());
    sidebar.append(&chats.search);
    let chat_scroll = gtk::ScrolledWindow::builder()
        .child(&chats.widget)
        .vexpand(true)
        .build();
    sidebar.append(&chat_scroll);
    split.set_start_child(Some(&sidebar));
    split.set_resize_start_child(false);
    split.set_shrink_start_child(false);

    let conversation = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let chat_header = ChatHeader::new();
    conversation.append(&chat_header.widget);
    let messages = MessageList::new(command_tx);
    conversation.append(&messages.scroller);
    let composer = Composer::new();
    conversation.append(&composer.widget);
    split.set_end_child(Some(&conversation));

    let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page.append(&split);
    (page, chats, chat_header, messages, composer, logout)
}

fn wire_logout(ui: &Ui) {
    let tx = ui.command_tx.clone();
    let status = ui.status.clone();
    ui.logout_button.connect_clicked(move |_| {
        if tx.try_send(BackendCommand::Logout).is_ok() {
            status.set_text("Logging out and clearing local TDLib data…");
        } else {
            status.set_text("Could not start logout. Try again.");
        }
    });
}

fn wire_auth(ui: &Ui) {
    let submit = {
        let ui = ui.clone();
        move || {
            let value = if matches!(*ui.current_prompt.borrow(), AuthPrompt::Password) {
                ui.auth_password.text().to_string()
            } else {
                ui.auth_input.text().to_string()
            };
            let command = match *ui.current_prompt.borrow() {
                AuthPrompt::PhoneNumber => BackendCommand::PhoneNumber(value),
                AuthPrompt::Code => BackendCommand::Code(value),
                AuthPrompt::Password => BackendCommand::Password(value),
                AuthPrompt::EmailAddress => BackendCommand::EmailAddress(value),
                AuthPrompt::EmailCode => BackendCommand::EmailCode(value),
                AuthPrompt::Registration => {
                    ui.status
                        .set_text("New account registration is not implemented in this prototype.");
                    return;
                }
                AuthPrompt::Other(ref state) => {
                    ui.status
                        .set_text(&format!("Unsupported authorization state: {state}"));
                    return;
                }
            };
            if ui.command_tx.try_send(command).is_err() {
                ui.status
                    .set_text("Backend is unavailable. Restart the client and try again.");
            } else {
                ui.status.set_text("Waiting for Telegram…");
            }
            ui.auth_input.set_text("");
            ui.auth_password.set_text("");
        }
    };
    let on_click = submit.clone();
    ui.auth_button.connect_clicked(move |_| on_click());
    let on_enter = submit;
    ui.auth_input.connect_activate(move |_| on_enter());

    let tx = ui.command_tx.clone();
    let status = ui.status.clone();
    ui.use_phone_button.connect_clicked(move |_| {
        if tx.try_send(BackendCommand::UsePhone).is_ok() {
            status.set_text("Switching to phone number sign-in…");
        } else {
            status.set_text("Could not switch sign-in method. Try again.");
        }
    });

    let tx = ui.command_tx.clone();
    let status = ui.status.clone();
    let qr_area = ui.qr_area.clone();
    ui.use_qr_button.connect_clicked(move |_| {
        if tx.try_send(BackendCommand::RequestQr).is_ok() {
            qr_area.set_visible(true);
            status.set_text("Requesting a QR code from Telegram…");
        } else {
            status.set_text("Could not request QR sign-in. Try again.");
        }
    });
}

fn wire_messages(ui: &Ui) {
    let tx = ui.command_tx.clone();
    let selected = ui.selected_chat.clone();
    let composer = ui.composer.input.clone();
    let send = move || {
        if let (Some(chat_id), text) = (selected.get(), composer.text().to_string()) {
            if !text.trim().is_empty()
                && tx
                    .try_send(BackendCommand::SendText { chat_id, text })
                    .is_ok()
            {
                composer.set_text("");
            }
        }
    };
    let on_click = send.clone();
    ui.composer.send.connect_clicked(move |_| on_click());
    ui.composer.input.connect_activate(move |_| send());

    let tx = ui.command_tx.clone();
    let selected = ui.selected_chat.clone();
    let messages = ui.messages.clone();
    let header = ui.chat_header.clone();
    let history = ui.history.clone();
    let ui_chat_list = ui.chats.clone();
    ui.chats.widget.connect_activate(move |_, position| {
        if let Some(chat) = ui_chat_list.chat_at(position) {
            let chat_id = chat.id;
            selected.set(Some(chat_id));
            header.set_chat(&chat);
            let request_id = history.borrow_mut().open_chat(ChatId(chat_id));
            messages.render(&history.borrow(), false);
            if tx
                .try_send(BackendCommand::OpenChat {
                    chat_id,
                    request_id,
                })
                .is_err()
            {
                history
                    .borrow_mut()
                    .fail_request(ChatId(chat_id), request_id);
            }
        }
    });

    let history = ui.history.clone();
    let tx = ui.command_tx.clone();
    let status = ui.status.clone();
    ui.messages
        .scroller
        .vadjustment()
        .connect_value_changed(move |adjustment| {
            let Some(chat_id) = history.borrow().chat_id() else {
                return;
            };
            let (direction, request) = if adjustment.value() <= 36.0 {
                (PageDirection::Older, history.borrow_mut().request_older())
            } else if adjustment.upper() - adjustment.page_size() - adjustment.value() <= 36.0 {
                (PageDirection::Newer, history.borrow_mut().request_newer())
            } else {
                return;
            };
            let Some((request_id, from_message_id)) = request else {
                return;
            };
            let command = match direction {
                PageDirection::Older => BackendCommand::LoadOlder {
                    chat_id: chat_id.0,
                    request_id,
                    from_message_id,
                },
                PageDirection::Newer => BackendCommand::LoadNewer {
                    chat_id: chat_id.0,
                    request_id,
                    from_message_id,
                },
                PageDirection::Initial => return,
            };
            if tx.try_send(command).is_err() {
                history.borrow_mut().fail_request(chat_id, request_id);
                status.set_text(
                    "Could not load another history page. Try scrolling to the edge again.",
                );
            }
        });
}

fn attach_events(ui: &Ui, events: async_channel::Receiver<BackendEvent>) {
    let ui = ui.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        let mut backend_error_shown = false;
        while let Ok(event) = events.recv().await {
            match event {
                BackendEvent::Authorization(AuthorizationState::Ready) => {
                    ui.stack.set_visible_child_name("main");
                    ui.status
                        .set_text("Connected. Messages are sent only after you press Send.");
                }
                BackendEvent::Authorization(state) => match state {
                    AuthorizationState::Closed => {
                        ui.stack.set_visible_child_name("login");
                        ui.chats.render(Vec::new(), None);
                        ui.selected_chat.set(None);
                        ui.history.borrow_mut().clear();
                        ui.messages.render(&ui.history.borrow(), false);
                        ui.status.set_text("Session closed. Preparing sign-in…");
                    }
                    AuthorizationState::LoggingOut => {
                        ui.status.set_text("Telegram is closing the session…");
                    }
                    AuthorizationState::WaitingForQrConfirmation => {
                        ui.status
                            .set_text("Scan this QR code with Telegram on your phone.");
                    }
                    AuthorizationState::WaitingForParameters => {
                        ui.status.set_text("Initializing the secure TDLib session…");
                    }
                    AuthorizationState::WaitingForPhoneNumber => {
                        ui.status.set_text("Preparing Telegram sign-in…");
                    }
                    AuthorizationState::WaitingForCode => {
                        ui.status.set_text("Waiting for the Telegram login code…");
                    }
                    AuthorizationState::WaitingForPassword => {
                        ui.status.set_text("Telegram requires your two-step verification password.");
                    }
                    AuthorizationState::Ready => ui.status.set_text("Connected to Telegram."),
                },
                BackendEvent::QrLink(link) => match render_qr(&ui.qr_area, &link) {
                    Ok(()) => {
                        ui.qr_area.set_visible(true);
                        ui.auth_input.set_visible(false);
                        ui.auth_password.set_visible(false);
                        ui.auth_button.set_visible(false);
                        ui.use_phone_button.set_visible(true);
                        ui.use_qr_button.set_visible(false);
                        ui.auth_title.set_text("Scan to sign in");
                    }
                    Err(()) => ui
                        .status
                        .set_text("Telegram returned an invalid QR login link."),
                },
                BackendEvent::Prompt(prompt) => set_prompt(&ui, prompt),
                BackendEvent::Chats(chats) => {
                    if let Some(selected) = ui.selected_chat.get() {
                        if let Some(chat) = chats.iter().find(|chat| chat.id == selected) {
                            ui.chat_header.set_chat(chat);
                        }
                    }
                    ui.chats.render(chats, ui.selected_chat.get());
                }
                BackendEvent::History { chat_id, request_id, messages, has_older, has_newer, direction } => {
                    let history_messages = messages.into_iter().map(from_backend_message).collect();
                    if ui.history.borrow_mut().apply_page(HistoryPage { chat_id: ChatId(chat_id), request_id, messages: history_messages, has_older, has_newer, direction }) {
                        ui.messages.render(&ui.history.borrow(), direction == PageDirection::Older);
                    }
                }
                BackendEvent::HistoryError { chat_id, request_id, message } => {
                    if ui.history.borrow_mut().fail_request(ChatId(chat_id), request_id) {
                        ui.status.set_text(&message);
                    }
                }
                BackendEvent::NewMessage { chat_id, message } => {
                    if ui.history.borrow_mut().upsert(ChatId(chat_id), from_backend_message(message)) {
                        ui.messages.render(&ui.history.borrow(), false);
                    }
                }
                BackendEvent::MessageEdited { chat_id, message } => {
                    if ui.history.borrow_mut().edit_content(ChatId(chat_id), message.id, message.text, message.kind) {
                        ui.messages.render(&ui.history.borrow(), false);
                    }
                }
                BackendEvent::MessageDeleted { chat_id, message_id } => {
                    if ui.history.borrow_mut().delete(ChatId(chat_id), message_id) {
                        ui.messages.render(&ui.history.borrow(), false);
                    }
                }
                BackendEvent::MessageDelivery { chat_id, message_id, delivery } => {
                    let delivery = match delivery {
                        telerust_tdlib_bridge::DeliveryState::Received => DeliveryState::Received,
                        telerust_tdlib_bridge::DeliveryState::Sending => DeliveryState::Sending,
                        telerust_tdlib_bridge::DeliveryState::Sent => DeliveryState::Sent,
                        telerust_tdlib_bridge::DeliveryState::Failed => DeliveryState::Failed,
                    };
                    if ui.history.borrow_mut().update_delivery(ChatId(chat_id), message_id, delivery) {
                        ui.messages.render(&ui.history.borrow(), false);
                    }
                }
                BackendEvent::FileUpdated { file_id, local_path, downloaded, downloading, downloaded_size } => {
                    if ui.history.borrow_mut().update_file(file_id, local_path, downloaded, downloading, downloaded_size) {
                        ui.messages.render(&ui.history.borrow(), false);
                    }
                }
                BackendEvent::FileDownloadFailed { file_id, message } => {
                    if ui.history.borrow_mut().update_file(file_id, None, false, false, 0) {
                        ui.messages.render(&ui.history.borrow(), false);
                    }
                    ui.status.set_text(&message);
                }
                BackendEvent::Status(text) => ui.status.set_text(&text),
                BackendEvent::Error(text) => {
                    backend_error_shown = true;
                    ui.status.set_text(&text);
                }
            }
        }
        if !backend_error_shown {
            ui.status.set_text(
                "TDLib stopped unexpectedly. Check the library and Secret Service, then restart Telerust.",
            );
        }
    });
}

fn set_prompt(ui: &Ui, prompt: AuthPrompt) {
    let (text, placeholder, password) = match prompt {
        AuthPrompt::PhoneNumber => (
            "Enter your phone number with country code",
            "+1 555 010 1234",
            false,
        ),
        AuthPrompt::Code => ("Enter the login code", "Login code", false),
        AuthPrompt::Password => (
            "Enter your two-step verification password",
            "Password",
            true,
        ),
        AuthPrompt::EmailAddress => ("Enter the account recovery email", "Email address", false),
        AuthPrompt::EmailCode => ("Enter the email verification code", "Email code", false),
        AuthPrompt::Registration => ("Account registration is not implemented", "", false),
        AuthPrompt::Other(ref state) => (
            "Unsupported Telegram authorization state",
            state.as_str(),
            false,
        ),
    };
    ui.auth_title.set_text(text);
    ui.qr_area.set_visible(false);
    ui.auth_input.set_placeholder_text(Some(placeholder));
    ui.auth_input.set_visible(!password);
    ui.auth_password.set_visible(password);
    ui.auth_button.set_visible(!matches!(
        prompt,
        AuthPrompt::Registration | AuthPrompt::Other(_)
    ));
    ui.use_phone_button.set_visible(false);
    ui.use_qr_button.set_visible(true);
    *ui.current_prompt.borrow_mut() = prompt;
    ui.status
        .set_text("Enter the requested value. The app does not log credentials.");
}

fn from_backend_message(message: MessageView) -> Message {
    Message {
        id: message.id,
        date: message.date,
        sender_id: message.sender_id,
        text: message.text,
        kind: message.kind,
        outgoing: message.outgoing,
        delivery: match message.delivery {
            telerust_tdlib_bridge::DeliveryState::Received => DeliveryState::Received,
            telerust_tdlib_bridge::DeliveryState::Sending => DeliveryState::Sending,
            telerust_tdlib_bridge::DeliveryState::Sent => DeliveryState::Sent,
            telerust_tdlib_bridge::DeliveryState::Failed => DeliveryState::Failed,
        },
    }
}

fn show_synthetic_history(ui: &Ui, count: usize) {
    let started = std::time::Instant::now();
    ui.stack.set_visible_child_name("main");
    ui.status.set_text(&format!(
        "Synthetic UI benchmark: requested {count}, displaying at most 1,000 messages. No Telegram session is opened."
    ));
    let synthetic_chat = ChatSummary {
        id: 7,
        title: "Synthetic conversation — тест ✨".into(),
        preview: "Deterministic local benchmark data".into(),
        sort_date: 1_700_000_000,
        is_channel: false,
        photo_file_id: None,
        photo_path: None,
    };
    ui.chats.render(vec![synthetic_chat.clone()], Some(7));
    ui.chat_header.set_chat(&synthetic_chat);
    let request_id = ui.history.borrow_mut().open_chat(ChatId(7));
    let show_media = std::env::var_os("TELERUST_SYNTHETIC_MEDIA").is_some();
    let switch_stress = std::env::var_os("TELERUST_SYNTHETIC_CHAT_SWITCHES").is_some();
    let messages = (0..count)
        .map(|index| {
            let kind = if show_media && index % 10 == 3 {
                MessageKind::Document(telerust_client_core::history::MediaFile {
                    id: 20_000 + index as i32,
                    name: format!("synthetic-report-{index}.pdf"),
                    mime_type: "application/pdf".into(),
                    size: 256 * 1024 + index as i64,
                    width: 0,
                    height: 0,
                    duration: 0,
                    local_path: None,
                    downloaded: false,
                    downloading: false,
                    downloaded_size: 0,
                })
            } else if show_media && index % 10 == 7 {
                MessageKind::Photo(telerust_client_core::history::MediaFile {
                    id: 30_000 + index as i32,
                    name: "Synthetic image".into(),
                    mime_type: "image/jpeg".into(),
                    size: 96 * 1024,
                    width: 320,
                    height: 240,
                    duration: 0,
                    local_path: None,
                    downloaded: false,
                    downloading: false,
                    downloaded_size: 0,
                })
            } else {
                MessageKind::Text
            };
            Message {
            id: index as i64 + 1,
            date: 1_700_000_000 + index as i64 * 60,
            sender_id: if index % 3 == 0 { "user:2" } else { "user:1" }.into(),
            text: match index % 4 {
                0 => format!("Русский тестовый текст {index} 🙂✨ — Unicode перенос строки проверяется на GTK."),
                1 => format!("Synthetic message {index}: {}", "long message ".repeat(24)),
                2 => format!("Short {index}"),
                _ => format!("Emoji sequence {index} 🚀 🌙 🦀 🎉"),
            },
            kind,
            outgoing: index % 3 != 0,
            delivery: DeliveryState::Sent,
        }})
        .collect();
    ui.history.borrow_mut().apply_page(HistoryPage {
        chat_id: ChatId(7),
        request_id,
        messages,
        has_older: false,
        has_newer: false,
        direction: PageDirection::Initial,
    });
    ui.messages.render(&ui.history.borrow(), false);
    eprintln!(
        "synthetic-history-model requested={} retained={} model_items={} model_build_ms={}",
        count,
        ui.history.borrow().messages().len(),
        ui.messages.store.n_items(),
        started.elapsed().as_millis()
    );
    let metrics = ui.messages.metrics.clone();
    let model = ui.messages.store.clone();
    let chat_list = ui.chats.widget.clone();
    let message_view = ui.messages.scroller.clone();
    let header = ui.chat_header.widget.clone();
    let composer = ui.composer.widget.clone();
    gtk::glib::timeout_add_seconds_local(2, move || {
        eprintln!(
            "synthetic-history model_items={} realized_rows={} peak_realized_rows={} created_rows={}",
            model.n_items(), metrics.realized(), metrics.peak_realized(), metrics.created()
        );
        eprintln!(
            "synthetic-geometry chat_list={}x{} message_view={}x{} header={}x{} composer={}x{}",
            chat_list.width(),
            chat_list.height(),
            message_view.width(),
            message_view.height(),
            header.width(),
            header.height(),
            composer.width(),
            composer.height()
        );
        gtk::glib::ControlFlow::Break
    });

    if switch_stress {
        let chats: Vec<_> = (0..8)
            .map(|index| ChatSummary {
                id: 7 + index,
                title: format!("Synthetic chat {index} — русский ✨"),
                preview: "Local chat switching fixture".into(),
                sort_date: 1_700_000_000 - index,
                is_channel: false,
                photo_file_id: None,
                photo_path: None,
            })
            .collect();
        ui.chats.render(chats.clone(), Some(7));
        let switch_ui = ui.clone();
        let message_count = count.min(1_000);
        gtk::glib::timeout_add_seconds_local(3, move || {
            let switch_ui = switch_ui.clone();
            let switch_chats = chats.clone();
            let switch_index = Rc::new(Cell::new(0_u32));
            let switch_counter = switch_index.clone();
            let started = std::time::Instant::now();
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(180), move || {
                let index = switch_counter.get();
                let chat = &switch_chats[index as usize % switch_chats.len()];
                switch_ui.selected_chat.set(Some(chat.id));
                switch_ui.chat_header.set_chat(chat);
                let request_id = switch_ui.history.borrow_mut().open_chat(ChatId(chat.id));
                let messages = (0..message_count)
                    .map(|message_index| Message {
                        id: message_index as i64 + 1,
                        date: 1_700_000_000 + message_index as i64 * 45,
                        sender_id: if message_index % 2 == 0 {
                            "user:synthetic-a"
                        } else {
                            "user:synthetic-b"
                        }
                        .into(),
                        text: format!(
                            "Chat {} synthetic message {} — emoji 🙂",
                            chat.id, message_index
                        ),
                        kind: MessageKind::Text,
                        outgoing: message_index % 2 == 1,
                        delivery: DeliveryState::Received,
                    })
                    .collect();
                switch_ui.history.borrow_mut().apply_page(HistoryPage {
                    chat_id: ChatId(chat.id),
                    request_id,
                    messages,
                    has_older: false,
                    has_newer: false,
                    direction: PageDirection::Initial,
                });
                switch_ui
                    .messages
                    .render(&switch_ui.history.borrow(), false);
                let next = index + 1;
                switch_counter.set(next);
                if next >= 12 {
                    eprintln!(
                        "synthetic-chat-switches count=12 messages_per_chat={} elapsed_ms={} retained={} model_items={} peak_realized_rows={} created_rows={}",
                        message_count,
                        started.elapsed().as_millis(),
                        switch_ui.history.borrow().messages().len(),
                        switch_ui.messages.store.n_items(),
                        switch_ui.messages.metrics.peak_realized(),
                        switch_ui.messages.metrics.created()
                    );
                    gtk::glib::ControlFlow::Break
                } else {
                    gtk::glib::ControlFlow::Continue
                }
            });
            gtk::glib::ControlFlow::Break
        });
        return;
    }

    let adjustment = ui.messages.scroller.vadjustment();
    let step = Rc::new(Cell::new(0_u32));
    let started = std::time::Instant::now();
    let scroll_step = step.clone();
    let scroll_metrics = ui.messages.metrics.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let current = scroll_step.get();
        let max = (adjustment.upper() - adjustment.page_size()).max(0.0);
        adjustment.set_value(if current % 2 == 0 { max } else { 0.0 });
        let next = current + 1;
        scroll_step.set(next);
        if next >= 40 {
            eprintln!(
                "synthetic-scroll traversals=40 elapsed_ms={} peak_realized_rows={} created_rows={}",
                started.elapsed().as_millis(),
                scroll_metrics.peak_realized(),
                scroll_metrics.created()
            );
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });
}

fn render_qr(area: &gtk::DrawingArea, link: &str) -> Result<(), ()> {
    let matrix = qr_matrix(link)?;
    area.set_draw_func(move |_, context, width, height| {
        let width = f64::from(width);
        let height = f64::from(height);
        let modules = matrix.len() + 8;
        let scale = (width.min(height) / modules as f64).floor().max(1.0);
        let size = modules as f64 * scale;
        let left = (width - size) / 2.0;
        let top = (height - size) / 2.0;
        context.set_source_rgb(1.0, 1.0, 1.0);
        let _ = context.paint();
        context.set_source_rgb(0.0, 0.0, 0.0);
        context.new_path();
        for (y, row) in matrix.iter().enumerate() {
            for (x, dark) in row.iter().enumerate() {
                if *dark {
                    context.rectangle(
                        left + (x + 4) as f64 * scale,
                        top + (y + 4) as f64 * scale,
                        scale,
                        scale,
                    );
                }
            }
        }
        let _ = context.fill();
    });
    area.queue_draw();
    Ok(())
}

fn qr_matrix(link: &str) -> Result<Vec<Vec<bool>>, ()> {
    if !link.starts_with("tg://login?token=") || link.len() > 4096 {
        return Err(());
    }
    let code = qrcode::QrCode::new(link.as_bytes()).map_err(|_| ())?;
    let width = code.width();
    Ok((0..width)
        .map(|y| {
            (0..width)
                .map(|x| code[(x, y)] == qrcode::Color::Dark)
                .collect()
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::qr_matrix;

    #[test]
    fn tdlib_login_link_becomes_a_square_qr_matrix() {
        let matrix = qr_matrix("tg://login?token=SYNTHETIC_TEST_TOKEN").unwrap();
        assert!(matrix.len() >= 21);
        assert!(matrix.iter().all(|row| row.len() == matrix.len()));
        assert!(matrix.iter().flatten().any(|dark| *dark));
    }

    #[test]
    fn qr_matrix_rejects_non_tdlib_links_and_oversized_values() {
        assert!(qr_matrix("https://example.invalid/QR").is_err());
        assert!(qr_matrix(&format!("tg://login?token={}", "x".repeat(4096))).is_err());
    }
}
