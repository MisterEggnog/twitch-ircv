use chrono::prelude::*;
use colored::Colorize;
use std::fmt;
use std::io;
use std::io::prelude::*;
use twitch_irc::message::Badge;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;

pub async fn message_handler<W: Write>(
    message: ServerMessage,
    start_time: DateTime<Utc>,
    out: &mut W,
) -> io::Result<bool> {
    let msg = match message {
        ServerMessage::Privmsg(msg) => print_chat_msg(msg, start_time, out).await,
        _ => Ok(()),
    };
    if let Err(err) = msg {
        if err.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("Write failed with {}", err);
            Err(err)
        } else {
            // Exit because pipe closed
            Ok(false)
        }
    } else {
        // Keep going
        Ok(true)
    }
}

async fn print_chat_msg<W: Write>(
    msg: PrivmsgMessage,
    start_time: DateTime<Utc>,
    out: &mut W,
) -> io::Result<()> {
    let time_since_start = msg.server_timestamp.signed_duration_since(start_time);
    let colored_name = match msg.name_color {
        Some(color) => msg.sender.name.truecolor(color.r, color.g, color.b),
        None => msg.sender.name.normal(),
    };
    let channel_badge = parse_badges(&msg.badges).await;
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
}

/// Broadcaster/Moderator/Vip
///
/// To my understanding these are mutulally exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChannelStatus {
    Broadcaster,
    Moderator,
    Vip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Subscriber {
    Month(i32),
    Founder,
}

/// The badges in a chat message.
///
/// This type is in development and may change.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
struct Badges {
    channel_status: Option<ChannelStatus>,
    /// This is the count of the badge, not the total months subbed.
    sub_badge_month: Option<Subscriber>,
    partner: bool,
    // staff: bool, TODO
}

async fn parse_badges(badges: &[Badge]) -> Badges {
    let mut channel_status = None;
    let mut sub_badge_month = None;
    let mut partner = false;
    for badge in badges {
        match badge.name.as_str() {
            "broadcaster" => channel_status = Some(ChannelStatus::Broadcaster),
            "moderator" => channel_status = Some(ChannelStatus::Moderator),
            "vip" => channel_status = Some(ChannelStatus::Vip),
            "subscriber" => sub_badge_month = badge.version.parse().ok().map(Subscriber::Month),
            "founder" => sub_badge_month = Some(Subscriber::Founder),
            "partner" => partner = true,
            // TODO "staff"
            _ => (),
        }
    }
    Badges {
        channel_status,
        sub_badge_month,
        partner,
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

impl fmt::Display for Badges {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Extend this with more checks as badges are added
        if self.partner {
            write!(f, "✅")?;
        }
        if let Some(ch) = &self.channel_status {
            write!(f, "{ch}")?;
        }
        Ok(())
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

#[tokio::test]
async fn does_not_panic_with_broken_pipe() -> io::Result<()> {
    use std::io;
    struct PanicsBrokenPipe;
    impl Write for PanicsBrokenPipe {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(From::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let message = ServerMessage::Privmsg(PrivmsgMessage::default());
    let start_time = Utc::now();
    let mut output = PanicsBrokenPipe;
    let res = message_handler(message, start_time, &mut output).await?;
    assert!(!res);
    Ok(())
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

#[test]
fn display_badges() {
    // TODO Add more cases
    //
    // We are NOT using default because we want to remember to update these tests
    // For each additional case.
    //
    // My idea is to make a iterator for every future case we add to the badges
    // Then zip those together and iterate / test each of those simultaneously.
    let statuses = [
        ChannelStatus::Broadcaster,
        ChannelStatus::Moderator,
        ChannelStatus::Vip,
    ]
    .into_iter()
    .map(|x| (Some(x), format!("{}", &x)))
    .chain(std::iter::once((None, "".to_string())));
    let partner_badge = [(false, ""), (true, "✅")].into_iter();
    let sub_badge_month = None;

    for ((status, status_expected), (partner, partner_expected)) in statuses.zip(partner_badge) {
        let badge = Badges {
            channel_status: status,
            sub_badge_month,
            partner,
        };
        let badge_str = badge.to_string();
        assert!(
            badge_str.contains(&status_expected),
            "Expected {badge:?} to hold {status_expected:?}"
        );
        assert!(badge_str.contains(partner_expected));
    }
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
    assert_eq!(sub_badge, Some(Subscriber::Month(90210)));

    let test_badges = [
        Badge {
            name: "partner".to_string(),
            version: "1".to_string(),
        },
        Badge {
            name: "west".to_string(),
            version: "90".to_string(),
        },
    ];
    let sub_badge = parse_badges(&test_badges).await;
    assert!(sub_badge.partner);

    let test_badges = [Badge {
        name: "founder".to_string(),
        version: "0".to_string(),
    }];
    let sub_badge = parse_badges(&test_badges).await;
    assert_eq!(Some(Subscriber::Founder), sub_badge.sub_badge_month);
}
