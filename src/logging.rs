use std::io::prelude::*;
use twitch_irc::message::AsRawIRC;
use twitch_irc::message::ServerMessage;

/// Log messages in IRC format
///
/// Logs PRIVMSG, USERNOTICE, CLEARCHAT, & CLEARMSG.
pub async fn log_v0<W: Write>(message: ServerMessage, out: &mut W) {}

#[tokio::test]
async fn log_v0_privmsg() {
    use twitch_irc::irc;
    use twitch_irc::message::PrivmsgMessage;

    let source = irc!["PRIVMSG", "#Orflex", "This is a real irc message, totes"];
    let expected = format!("{}\n", source.as_raw_irc());

    let fake_privmsg = PrivmsgMessage {
        source,
        ..Default::default()
    };
    let fake_privmsg = ServerMessage::Privmsg(fake_privmsg);

    let mut output = vec![];
    log_v0(fake_privmsg, &mut output).await;
    let output = String::from_utf8(output).unwrap();

    assert_eq!(output, expected);
}
