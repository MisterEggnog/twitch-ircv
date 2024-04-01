use std::env;
use std::error::Error;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

pub async fn main_prime<F>(func: F) -> Result<(), Box<dyn Error>>
where
    F: Fn(ServerMessage) + std::marker::Send + 'static,
{
    let channel = env::args()
        .nth(1)
        .expect("Must have channel name as first arg");

    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            func(message);
        }
    });

    client.join(channel)?;
    join_handle.await?;
    Ok(())
}
