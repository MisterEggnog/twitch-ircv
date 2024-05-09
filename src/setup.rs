use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

pub type TwitchClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;

/// Simplified version of TwitchIRCClient::new with default config
pub fn build_irc_client() -> (UnboundedReceiver<ServerMessage>, TwitchClient) {
    let config = ClientConfig::default();
    TwitchClient::new(config)
}
