use std::io::stdout;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

use crate::args::Args;
use crate::chat_logger::message_handler;

pub type TwitchClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;

pub async fn init(args: Args) {
    let (incoming_messages, client) = build_irc_client();
    let join_handle = setup_fancy_output(incoming_messages);

    client.join(args.channel_name).unwrap();

    join_handle.await.unwrap();
}

fn receiver_splitter(
    mut incoming: UnboundedReceiver<ServerMessage>,
) -> (
    JoinHandle<()>,
    UnboundedReceiver<ServerMessage>,
    UnboundedReceiver<ServerMessage>,
) {
    use tokio::sync::mpsc;
    let (tx1, rx1) = mpsc::unbounded_channel();
    let (tx2, rx2) = mpsc::unbounded_channel();
    let handle = tokio::spawn(async move {
        while let Some(message) = incoming.recv().await {
            tx1.send(message.clone()).unwrap();
            tx2.send(message).unwrap();
        }
    });
    (handle, rx1, rx2)
}

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

#[tokio::test]
async fn receiver_splitter_is_balanced() {
    use tokio::sync::mpsc;
    use twitch_irc::message::PrivmsgMessage;
    let (tx, rx) = mpsc::unbounded_channel();
    let (handle, mut out1, mut out2) = receiver_splitter(rx);
    let test_msg = ServerMessage::Privmsg(PrivmsgMessage::default());

    tx.send(test_msg.clone()).unwrap();
    let res1 = out1.recv().await.unwrap();
    let res2 = out2.recv().await.unwrap();

    // == NOt ImPlEMENted FoR seRVERmesSage
    let res1 = match res1 {
        ServerMessage::Privmsg(msg) => msg,
        _ => unreachable!(),
    };
    let res2 = match res2 {
        ServerMessage::Privmsg(msg) => msg,
        _ => unreachable!(),
    };
    let test_msg = match test_msg {
        ServerMessage::Privmsg(msg) => msg,
        _ => unreachable!(),
    };
    assert_eq!(test_msg, res1);
    assert_eq!(test_msg, res2);

    drop(tx);
    handle.await.unwrap();
}
