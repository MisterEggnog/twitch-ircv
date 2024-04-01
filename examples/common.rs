use std::env;
use std::error::Error;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

pub async fn main_prime() -> Result<(), Box<dyn Error>> {
    let channel = env::args()
        .nth(1)
        .expect("Must have channel name as first arg");

    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            if let ServerMessage::Privmsg(message) = message {
                println!("{} {:?}", message.sender.name, message.badges);
            }
        }
    });

    client.join(channel)?;
    join_handle.await?;
    Ok(())
}
