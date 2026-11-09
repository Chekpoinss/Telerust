use adw::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use telerust_client_core::{
    events::AuthorizationState,
    settings::{config_path, Settings},
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
    current_prompt: Rc<RefCell<AuthPrompt>>,
    chats: gtk::ListBox,
    messages: gtk::ListBox,
    composer: gtk::Entry,
    send_button: gtk::Button,
    selected_chat: Rc<Cell<Option<i64>>>,
    command_tx: async_channel::Sender<BackendCommand>,
}

fn main() {
    let application = adw::Application::builder()
        .application_id("org.telerust.Client")
        .build();
    application.connect_activate(build_window);
    application.run();
}

fn build_window(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Unofficial Telerust")
        .default_width(1020)
        .default_height(700)
        .build();

    let (command_tx, event_rx) = start_backend();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let status = gtk::Label::new(None);
    status.set_xalign(0.0);
    status.set_margin_start(12);
    status.set_margin_end(12);
    status.set_margin_top(6);
    status.set_margin_bottom(6);
    status.add_css_class("dim-label");
    content.append(&status);

    let stack = gtk::Stack::builder()
        .hexpand(true)
        .vexpand(true)
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    content.append(&stack);

    let (login_page, auth_title, auth_input, auth_password, auth_button) = login_view();
    stack.add_named(&login_page, Some("login"));
    let main_page = main_view();
    stack.add_named(&main_page.0, Some("main"));
    stack.set_visible_child_name("login");
    window.set_content(Some(&content));

    let ui = Ui {
        stack,
        status,
        auth_title,
        auth_input,
        auth_password,
        auth_button,
        current_prompt: Rc::new(RefCell::new(AuthPrompt::PhoneNumber)),
        chats: main_page.1,
        messages: main_page.2,
        composer: main_page.3,
        send_button: main_page.4,
        selected_chat: Rc::new(Cell::new(None)),
        command_tx,
    };

    wire_auth(&ui);
    wire_messages(&ui);
    attach_events(&ui, event_rx);

    let command_tx = ui.command_tx.clone();
    app.connect_shutdown(move |_| {
        let _ = command_tx.try_send(BackendCommand::Shutdown);
    });
    window.present();
}

fn start_backend() -> (
    async_channel::Sender<BackendCommand>,
    async_channel::Receiver<BackendEvent>,
) {
    match Settings::load() {
        Ok(settings) => telerust_tdlib_bridge::spawn(settings),
        Err(error) => {
            let (command_tx, _command_rx) = async_channel::bounded(1);
            let (event_tx, event_rx) = async_channel::bounded(8);
            let message = match error {
                telerust_client_core::settings::SettingsError::Missing => format!(
                    "Create {} with api_id and api_hash. Start with use_test_dc = true.",
                    config_path().join("config.toml").display()
                ),
                telerust_client_core::settings::SettingsError::InsecurePermissions =>
                    "Configuration permissions must be 0600; run chmod 600 on config.toml.".to_string(),
                telerust_client_core::settings::SettingsError::Invalid =>
                    "Configuration is invalid. Check api_id and api_hash; values are never shown here.".to_string(),
            };
            let _ = event_tx.try_send(BackendEvent::Error(message));
            (command_tx, event_rx)
        }
    }
}

fn login_view() -> (gtk::Box, gtk::Label, gtk::Entry, gtk::Entry, gtk::Button) {
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
    let disclosure = gtk::Label::new(Some("An unofficial client using the Telegram API. Development is configured for Telegram test servers by default."));
    disclosure.set_wrap(true);
    disclosure.set_margin_top(8);
    disclosure.set_margin_bottom(20);
    outer.append(&disclosure);

    let prompt = gtk::Label::new(Some("Loading TDLib authorization state…"));
    prompt.set_xalign(0.0);
    outer.append(&prompt);
    let input = gtk::Entry::new();
    input.set_placeholder_text(Some("Phone number"));
    input.set_margin_top(8);
    outer.append(&input);
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
    (outer, prompt, input, password, button)
}

// Return page plus widgets which need event updates.
fn main_view() -> (
    gtk::Box,
    gtk::ListBox,
    gtk::ListBox,
    gtk::Entry,
    gtk::Button,
) {
    let split = gtk::Paned::new(gtk::Orientation::Horizontal);
    split.set_position(320);
    split.set_wide_handle(true);

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let heading = gtk::Label::new(Some("Chats"));
    heading.add_css_class("title-2");
    heading.set_xalign(0.0);
    heading.set_margin_start(12);
    heading.set_margin_top(12);
    heading.set_margin_bottom(8);
    sidebar.append(&heading);
    let chats = gtk::ListBox::new();
    chats.set_selection_mode(gtk::SelectionMode::Single);
    chats.add_css_class("boxed-list");
    let chat_scroll = gtk::ScrolledWindow::builder()
        .child(&chats)
        .vexpand(true)
        .build();
    sidebar.append(&chat_scroll);
    split.set_start_child(Some(&sidebar));
    split.set_resize_start_child(false);
    split.set_shrink_start_child(false);

    let conversation = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let messages = gtk::ListBox::new();
    messages.set_selection_mode(gtk::SelectionMode::None);
    messages.set_valign(gtk::Align::End);
    let message_scroll = gtk::ScrolledWindow::builder()
        .child(&messages)
        .vexpand(true)
        .build();
    conversation.append(&message_scroll);
    let composer_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    composer_row.set_margin_start(12);
    composer_row.set_margin_end(12);
    composer_row.set_margin_top(8);
    composer_row.set_margin_bottom(12);
    let composer = gtk::Entry::new();
    composer.set_hexpand(true);
    composer.set_placeholder_text(Some("Message"));
    let send = gtk::Button::with_label("Send");
    send.add_css_class("suggested-action");
    composer_row.append(&composer);
    composer_row.append(&send);
    conversation.append(&composer_row);
    split.set_end_child(Some(&conversation));

    let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page.append(&split);
    (page, chats, messages, composer, send)
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
}

fn wire_messages(ui: &Ui) {
    let tx = ui.command_tx.clone();
    let selected = ui.selected_chat.clone();
    let composer = ui.composer.clone();
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
    ui.send_button.connect_clicked(move |_| on_click());
    ui.composer.connect_activate(move |_| send());

    let tx = ui.command_tx.clone();
    let selected = ui.selected_chat.clone();
    let messages = ui.messages.clone();
    ui.chats.connect_row_activated(move |_, row| {
        if let Ok(chat_id) = row.widget_name().parse::<i64>() {
            selected.set(Some(chat_id));
            clear_list(&messages);
            append_info(&messages, "Loading recent messages…");
            if tx.try_send(BackendCommand::OpenChat(chat_id)).is_err() {
                clear_list(&messages);
                append_info(&messages, "Could not request chat history.");
            }
        }
    });
}

fn attach_events(ui: &Ui, events: async_channel::Receiver<BackendEvent>) {
    let ui = ui.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = events.recv().await {
            match event {
                BackendEvent::Authorization(AuthorizationState::Ready) => {
                    ui.stack.set_visible_child_name("main");
                    ui.status
                        .set_text("Connected. Messages are sent only after you press Send.");
                }
                BackendEvent::Authorization(state) => {
                    ui.status.set_text(match state {
                        AuthorizationState::Closed => "TDLib session closed.",
                        _ => "Connecting to Telegram test servers…",
                    });
                }
                BackendEvent::Prompt(prompt) => set_prompt(&ui, prompt),
                BackendEvent::Chats(chats) => render_chats(
                    &ui.chats,
                    chats,
                    &ui.command_tx,
                    &ui.selected_chat,
                    &ui.messages,
                ),
                BackendEvent::History { chat_id, messages } => {
                    if ui.selected_chat.get() == Some(chat_id) {
                        render_messages(&ui.messages, messages);
                    }
                }
                BackendEvent::NewMessage { chat_id, message } => {
                    if ui.selected_chat.get() == Some(chat_id) {
                        append_message(&ui.messages, &message);
                    }
                }
                BackendEvent::Status(text) => ui.status.set_text(&text),
                BackendEvent::Error(text) => ui.status.set_text(&text),
            }
        }
        ui.status.set_text("Backend stopped.");
    });
}

fn set_prompt(ui: &Ui, prompt: AuthPrompt) {
    let (text, placeholder, password) = match prompt {
        AuthPrompt::PhoneNumber => ("Enter the test account phone number", "Phone number", false),
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
    ui.auth_input.set_placeholder_text(Some(placeholder));
    ui.auth_input.set_visible(!password);
    ui.auth_password.set_visible(password);
    *ui.current_prompt.borrow_mut() = prompt;
    ui.status
        .set_text("Enter the requested value. The app does not log credentials.");
}

fn render_chats(
    list: &gtk::ListBox,
    chats: Vec<ChatSummary>,
    tx: &async_channel::Sender<BackendCommand>,
    selected: &Rc<Cell<Option<i64>>>,
    messages: &gtk::ListBox,
) {
    let selected_id = selected.get();
    clear_list(list);
    for chat in chats {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(chat.id.to_string().as_str());
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 2);
        box_.set_margin_start(12);
        box_.set_margin_end(12);
        box_.set_margin_top(9);
        box_.set_margin_bottom(9);
        let title = gtk::Label::new(Some(&chat.title));
        title.set_xalign(0.0);
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title.add_css_class("heading");
        let preview = gtk::Label::new(Some(&chat.preview));
        preview.set_xalign(0.0);
        preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
        preview.add_css_class("dim-label");
        box_.append(&title);
        box_.append(&preview);
        row.set_child(Some(&box_));
        list.append(&row);
        if selected_id == Some(chat.id) {
            list.select_row(Some(&row));
        }
    }
    let _ = (tx, messages); // Row activation owns the command path; keep this render function pure.
}

fn render_messages(list: &gtk::ListBox, messages: Vec<MessageView>) {
    clear_list(list);
    for message in messages.into_iter().rev() {
        append_message(list, &message);
    }
}

fn append_message(list: &gtk::ListBox, message: &MessageView) {
    let row = gtk::ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    line.set_margin_start(12);
    line.set_margin_end(12);
    line.set_margin_top(4);
    line.set_margin_bottom(4);
    line.set_halign(if message.outgoing {
        gtk::Align::End
    } else {
        gtk::Align::Start
    });
    let bubble = gtk::Label::new(Some(&message.text));
    bubble.set_wrap(true);
    bubble.set_max_width_chars(72);
    bubble.set_xalign(0.0);
    bubble.set_margin_start(10);
    bubble.set_margin_end(10);
    bubble.set_margin_top(7);
    bubble.set_margin_bottom(7);
    bubble.add_css_class(if message.outgoing {
        "card"
    } else {
        "boxed-list"
    });
    line.append(&bubble);
    row.set_child(Some(&line));
    list.append(&row);
}

fn append_info(list: &gtk::ListBox, text: &str) {
    let label = gtk::Label::new(Some(text));
    label.set_margin_top(12);
    label.set_margin_bottom(12);
    let row = gtk::ListBoxRow::new();
    row.set_child(Some(&label));
    list.append(&row);
}

fn clear_list(list: &gtk::ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}
