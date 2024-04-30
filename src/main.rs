use std::io::stdout;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

mod args;
mod badges;
mod chat_logger;
mod logging;
use chat_logger::*;

#[tokio::main]
async fn main() {
    let args: args::Args = argh::from_env();
    let channel = args.channel_name;

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
