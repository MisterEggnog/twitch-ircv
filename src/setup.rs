use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;

/// Simplified version of TwitchIRCClient::new with default config
fn setup_irc_client(channel: &str) -> UnboundedReceiver<ServerMessage> {
    todo!()
}
