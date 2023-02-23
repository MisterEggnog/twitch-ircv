use std::env;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

#[tokio::main]
async fn main() {
    let channel = env::args()
        .nth(1)
        .expect("Must have channel name as first arg");

    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            println!("{:?}", message);
        }
    });

    client.join(channel).unwrap();

    join_handle.await.unwrap();
}
