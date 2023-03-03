use chrono::prelude::*;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;

pub async fn message_handler(message: ServerMessage, start_time: DateTime<Utc>) {
    match message {
        ServerMessage::Privmsg(msg) => print_chat_msg(msg, start_time).await,
        _ => (),
    }
}

async fn print_chat_msg(msg: PrivmsgMessage, start_time: DateTime<Utc>) {
    let time_since_start = msg.server_timestamp.signed_duration_since(start_time);
    println!(
        "{:02}:{:02}:{:02} {}: {}",
        time_since_start.num_hours(),
        time_since_start.num_minutes(),
        time_since_start.num_seconds(),
        msg.sender.name,
        msg.message_text
    );
}

#[tokio::test]
async fn print_chat_msg_test() {
    use chrono::Duration;
    use twitch_irc::irc;
    use twitch_irc::message::TwitchUserBasics;

    let start_time = Utc::now();
    let time_offset = Duration::hours(11) + Duration::minutes(11) + Duration::seconds(11);
    let message_time = start_time + time_offset;

    let sender_name = "snapdragon".to_string();
    let sender = TwitchUserBasics {
        name: sender_name.clone(),
        id: "".to_owned(),
        login: "".to_owned(),
    };
    let message_contents = "AAAAAAAAAAAAAAAAAA.";

    /*let irc_msg = irc![
        "PRIVMSG",
        "#channel",
        sender_name,
        message_time,
        message_contents
    ];*/
    //panic!("{:?}", irc_msg);
}
