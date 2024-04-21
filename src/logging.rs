use std::io::prelude::*;
use twitch_irc::message::ServerMessage;

/// Log messages in IRC format
///
/// Logs PRIVMSG, USERNOTICE, CLEARCHAT, & CLEARMSG.
pub async fn log_v0<W: Write>(message: ServerMessage, out: &mut W) {}

#[tokio::test]
async fn log_v0_tests() {}
