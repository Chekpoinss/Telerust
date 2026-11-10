use async_channel::{Receiver, Sender};
use telerust_client_core::settings::{Settings, SettingsError};
use telerust_tdlib_bridge::{BackendCommand, BackendEvent};

pub struct TelegramService {
    pub commands: Sender<BackendCommand>,
    pub events: Receiver<BackendEvent>,
}

pub fn start() -> TelegramService {
    match Settings::load() {
        Ok(settings) => {
            let (commands, events) = telerust_tdlib_bridge::spawn(settings);
            TelegramService { commands, events }
        }
        Err(error) => {
            let (commands, _receiver) = async_channel::bounded(1);
            let (sender, events) = async_channel::bounded(8);
            let message = match error {
                SettingsError::Missing => "Set up your API credentials from https://my.telegram.org/apps using the button below.",
                SettingsError::InsecurePermissions => "Configuration permissions must be 0600; run chmod 600 on config.toml.",
                SettingsError::Invalid => "Configuration is invalid. Check api_id and api_hash; values are never shown here.",
            };
            let _ = sender.try_send(BackendEvent::Error(message.to_string()));
            TelegramService { commands, events }
        }
    }
}
