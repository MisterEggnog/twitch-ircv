use chrono::prelude::*;
use std::io::prelude::*;
use std::io::stdout;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;

pub async fn message_handler(message: ServerMessage, start_time: DateTime<Utc>) {
    match message {
        ServerMessage::Privmsg(msg) => print_chat_msg(msg, start_time, &mut stdout()).await,
        _ => (),
    }
}

async fn print_chat_msg<W: Write>(msg: PrivmsgMessage, start_time: DateTime<Utc>, out: &mut W) {
    let time_since_start = msg.server_timestamp.signed_duration_since(start_time);
    writeln!(
        out,
        "{:02}:{:02}:{:02} {}: {}",
        time_since_start.num_hours(),
        time_since_start.num_minutes() % 60,
        time_since_start.num_seconds() % 60,
        msg.sender.name,
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
    let message_text_ = "AAAAAAAAAAAAAAAAAA.".to_string();

    let message = PrivmsgMessage {
        sender,
        server_timestamp: message_time,
        message_text: message_text_.clone(),
        ..Default::default()
    };

    let mut output = vec![];

    print_chat_msg(message, start_time, &mut output).await;
    assert_eq!(
        output,
        format!("11:11:11 {sender_name}: {message_text_}\n").into_bytes(),
        "\n{}",
        String::from_utf8_lossy(&output)
    );
}
