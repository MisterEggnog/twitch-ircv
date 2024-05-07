use std::io::stdout;
use std::sync::mpsc::Sender;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

use crate::chat_logger::*;

enum IRCSink {
    Stdout(Sender<ServerMessage>),
}

pub async fn setup_irc_client<F>(channel: String, message_dest: F)
where
    F: Fn(ServerMessage),
{
    let startup_time = chrono::Utc::now();
    println!("Logging started at {}", startup_time);

    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let mut stdout = stdout();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            if !message_handler(message, startup_time, &mut stdout)
                .await
                .expect("Failed to write message")
            {
                break;
            }
        }
    });

    client.join(channel).unwrap();

    join_handle.await.unwrap();
}
