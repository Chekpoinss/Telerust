use adw::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use telerust_client_core::{events::AuthorizationState, settings::Settings};
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
        qr_area,
        use_phone_button,
        use_qr_button,
        credentials_button,
        logout_button: main_page.5,
        current_prompt: Rc::new(RefCell::new(AuthPrompt::PhoneNumber)),
        chats: main_page.1,
        messages: main_page.2,
        composer: main_page.3,
        send_button: main_page.4,
        selected_chat: Rc::new(Cell::new(None)),
        command_tx,
    };

    wire_auth(&ui);
    wire_credentials(&ui, &window);
    wire_messages(&ui);
    wire_logout(&ui);
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
                telerust_client_core::settings::SettingsError::Missing => "Set up your API credentials from https://my.telegram.org/apps using the button below.".to_string(),
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
fn main_view() -> (
    gtk::Box,
    gtk::ListBox,
    gtk::ListBox,
    gtk::Entry,
    gtk::Button,
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
    heading_row.append(&heading);
    heading_row.append(&logout);
    sidebar.append(&heading_row);
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
    (page, chats, messages, composer, send, logout)
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
                        clear_list(&ui.chats);
                        clear_list(&ui.messages);
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
