use std::io::stdout;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

use crate::chat_logger::message_handler;

pub type TwitchClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;

/// Simplified version of TwitchIRCClient::new with default config
pub fn build_irc_client() -> (UnboundedReceiver<ServerMessage>, TwitchClient) {
    let config = ClientConfig::default();
    TwitchClient::new(config)
}

pub fn setup_fancy_output(mut incoming: UnboundedReceiver<ServerMessage>) -> JoinHandle<()> {
    let startup_time = chrono::Utc::now();
    println!("Logging started at {}", startup_time);
    let mut stdout = stdout();

    tokio::spawn(async move {
        while let Some(message) = incoming.recv().await {
            if !message_handler(message, startup_time, &mut stdout)
                .await
                .expect("Failed to write message")
            {
                break;
            }
        }
    })
}
