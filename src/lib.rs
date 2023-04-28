use chrono::prelude::*;
use colored::Colorize;
use std::io::prelude::*;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;

pub async fn message_handler<W: Write>(
    message: ServerMessage,
    start_time: DateTime<Utc>,
    out: &mut W,
) {
    match message {
        ServerMessage::Privmsg(msg) => print_chat_msg(msg, start_time, out).await,
        _ => (),
    }
}

async fn print_chat_msg<W: Write>(msg: PrivmsgMessage, start_time: DateTime<Utc>, out: &mut W) {
    let time_since_start = msg.server_timestamp.signed_duration_since(start_time);
    let colored_name = match msg.name_color {
        Some(color) => msg.sender.name.truecolor(color.r, color.g, color.b),
        None => msg.sender.name.normal(),
    };
    writeln!(
        out,
        "{:02}:{:02}:{:02} {}: {}",
        time_since_start.num_hours(),
        time_since_start.num_minutes() % 60,
        time_since_start.num_seconds() % 60,
        colored_name,
        msg.message_text
    )
    .expect("Not going to bother to check this lol");
}

#[tokio::test]
async fn print_chat_msg_test() {
    use chrono::Duration;
    use twitch_irc::message::TwitchUserBasics;

    let start_time = Utc::now();
    let time_offset = Duration::hours(11) + Duration::minutes(11) + Duration::seconds(11);
    let message_time = start_time + time_offset;

    let sender_name = "snapdragon".to_string();
    let sender = TwitchUserBasics {
        name: sender_name.clone(),
        ..Default::default()
    };
    let message_str = "Bannana bread";
    let message_text = String::from(message_str);

    let message = PrivmsgMessage {
        sender,
        server_timestamp: message_time,
        message_text,
        ..Default::default()
    };

    let mut output = vec![];

    print_chat_msg(message, start_time, &mut output).await;
    assert_eq!(
        output,
        format!("11:11:11 {sender_name}: {message_str}\n").into_bytes(),
        "\n{}",
        String::from_utf8_lossy(&output)
    );
}
