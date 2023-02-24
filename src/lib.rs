use chrono::{DateTime, Utc};
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;

pub async fn message_handler(message: ServerMessage, start_time: &DateTime<Utc>) {
    match message {
        ServerMessage::Privmsg(msg) => print_chat_msg(msg, &start_time).await,
        _ => (),
    }
}

async fn print_chat_msg(msg: PrivmsgMessage, start_time: &DateTime<Utc>) {
    let time_since_start = msg.server_timestamp.signed_duration_since(*start_time);
    println!(
        "{} {}: {}",
        time_since_start, msg.sender.name, msg.message_text
    );
}
