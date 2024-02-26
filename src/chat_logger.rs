use chrono::prelude::*;
use colored::Colorize;
use std::fmt;
use std::io::prelude::*;
use twitch_irc::message::Badge;
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
    let channel_badge = parse_badges(&msg.badges).await;
    let channel_badge = channel_badge.channel_status;
    let channel_badge = channel_badge.map_or("".to_string(), |s| format!("{}", s));
    writeln!(
        out,
        "{:02}:{:02}:{:02} {}{}: {}",
        time_since_start.num_hours(),
        time_since_start.num_minutes() % 60,
        time_since_start.num_seconds() % 60,
        channel_badge,
        colored_name,
        msg.message_text
    )
    .expect("Not going to bother to check this lol");
}

#[derive(Debug, PartialEq)]
enum ChannelStatus {
    Broadcaster,
    Moderator,
    Vip,
}

#[derive(Default, Debug, PartialEq)]
struct Badges {
    channel_status: Option<ChannelStatus>,
    sub_badge_month: Option<i32>,
    // partner: bool, TODO
    // staff: bool, TODO
}

async fn parse_badges(badges: &[Badge]) -> Badges {
    let mut channel_status = None;
    let mut sub_badge_month = None;
    for badge in badges {
        match badge.name.as_str() {
            "broadcaster" => channel_status = Some(ChannelStatus::Broadcaster),
            "moderator" => channel_status = Some(ChannelStatus::Moderator),
            "vip" => channel_status = Some(ChannelStatus::Vip),
            "subscriber" => sub_badge_month = badge.version.parse().ok(),
            // TODO "partner"
            // TODO "staff"
            _ => (),
        }
    }
    Badges {
        channel_status,
        sub_badge_month,
    }
}

impl fmt::Display for ChannelStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ChannelStatus::Broadcaster => "📹",
                ChannelStatus::Moderator => "🗡️",
                ChannelStatus::Vip => "💎",
            }
        )
    }
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

#[test]
fn display_channel_status() {
    let statuses = [
        ChannelStatus::Broadcaster,
        ChannelStatus::Moderator,
        ChannelStatus::Vip,
    ]
    .map(|x| format!("{}", x));
    assert_ne!(statuses[0], statuses[1]);
    assert_ne!(statuses[1], statuses[2]);
    assert_ne!(statuses[0], statuses[2]);
}

#[tokio::test]
async fn test_parse_badges() {
    let valid_strings = ["broadcaster", "moderator", "vip"];
    for s in valid_strings {
        let test_badges = [
            Badge {
                name: "beep".to_string(),
                version: "6".to_string(),
            },
            Badge {
                name: s.to_string(),
                version: "1".to_string(),
            },
        ];
        let permission_badge = parse_badges(&test_badges).await;
        let permission_badge = permission_badge.channel_status;
        assert!(permission_badge.is_some());
    }

    let test_badges = [
        Badge {
            name: "adsad".to_string(),
            version: "202".to_string(),
        },
        Badge {
            name: "subscriber".to_string(),
            version: "90210".to_string(),
        },
    ];
    let sub_badge = parse_badges(&test_badges).await;
    let sub_badge = sub_badge.sub_badge_month;
    assert_eq!(sub_badge, Some(90210));
}
