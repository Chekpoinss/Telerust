pub fn install() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(
        "window.telerust-window { background: #17212b; color: #f5f5f5; }\
         .chat-list { background: #17212b; }\
         .chat-list row { border-bottom: 1px solid alpha(#8a9aa8, 0.12); padding: 2px; }\
         .chat-list row:hover { background: #202b36; }\
         .chat-list row:selected { background: #2b5278; }\
         .chat-header { background: #17212b; padding: 10px 14px; border-bottom: 1px solid alpha(#8a9aa8, 0.18); }\
         .avatar-placeholder { min-width: 42px; min-height: 42px; border-radius: 999px; background: #5288c1; color: white; font-size: 24px; }\
         .message-in { background: #182533; border-radius: 14px 14px 14px 4px; }\
         .message-out { background: #2b5278; border-radius: 14px 14px 4px 14px; }\
         .message-grouped-in { border-radius: 8px 14px 14px 4px; }\
         .message-grouped-out { border-radius: 14px 8px 4px 14px; }\
         .date-separator { background: alpha(#182533, 0.88); border-radius: 999px; padding: 4px 10px; }\
         .message-time { font-size: 0.75em; opacity: 0.7; }",
    );
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
