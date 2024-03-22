use std::env;
use std::io::stdout;
use std::process::ExitCode;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

mod badges;
mod chat_logger;
use chat_logger::*;

#[tokio::main]
async fn main() -> ExitCode {
    let channel = match env::args().nth(1) {
        Some(n) => n,
        None => {
            eprintln!("Must have channel name as first arg");
            return ExitCode::FAILURE;
        }
    };

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

    ExitCode::SUCCESS
}
