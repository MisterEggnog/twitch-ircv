use std::io;
use std::io::prelude::*;
use twitch_irc::message::AsRawIRC;
use twitch_irc::message::ServerMessage;

/// Log messages in IRC format
///
/// Logs PRIVMSG, USERNOTICE, CLEARCHAT, & CLEARMSG.
pub async fn log_v0<W: Write>(message: ServerMessage, out: &mut W) -> io::Result<()> {
    match message {
        ServerMessage::Privmsg(msg) => writeln!(out, "{}", msg.source.as_raw_irc()),
        ServerMessage::UserNotice(msg) => writeln!(out, "{}", msg.source.as_raw_irc()),
        ServerMessage::ClearChat(msg) => writeln!(out, "{}", msg.source.as_raw_irc()),
        ServerMessage::ClearMsg(msg) => writeln!(out, "{}", msg.source.as_raw_irc()),
        _ => Ok(()),
    }?;

    Ok(())
}

#[tokio::test]
async fn log_v0_privmsg() -> io::Result<()> {
    use twitch_irc::irc;
    use twitch_irc::message::PrivmsgMessage;

    let source = irc!["PRIVMSG", "#Orflex", "This is a real irc message, totes"];
    let expected = format!("{}\n", source.as_raw_irc());

    let example = crate::setup::make_privmsg_example();
    let fake_privmsg = PrivmsgMessage { source, ..example };
    let fake_privmsg = ServerMessage::Privmsg(fake_privmsg);

    let mut output = vec![];
    let _ = log_v0(fake_privmsg, &mut output).await?;
    let output = String::from_utf8(output).unwrap();

    assert_eq!(output, expected);

    Ok(())
}
