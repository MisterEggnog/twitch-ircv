use std::fmt;
use twitch_irc::message::Badge;

/// Broadcaster/Moderator/Vip
///
/// To my understanding these are mutulally exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelStatus {
    Broadcaster,
    Moderator,
    Vip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subscriber {
    Month(i32),
    Founder,
}

/// The badges in a chat message.
///
/// This type is in development and may change.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Badges {
    pub channel_status: Option<ChannelStatus>,
    /// This is the count of the badge, not the total months subbed.
    pub sub_badge_month: Option<Subscriber>,
    pub partner: bool,
    // staff: bool, TODO
}

pub async fn parse_badges(badges: &[Badge]) -> Badges {
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
