use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;

use twitch_irc::message::{IRCMessage, ServerMessage};

use twitch_chat_logger::logging::log_v0;

fn valid_irc(s: &str) -> bool {
    s.contains("PRIVMSG")
        || s.contains("USERNOTICE")
        || s.contains("CLEARCHAT")
        || s.contains("CLEARMSG")
}

#[tokio::test]
async fn test_no_ping() -> Result<(), Box<dyn Error>> {
    let f = File::open("tests/irc_data_no_ping")?;
    let f = BufReader::new(f);
    let irc_lines: Vec<String> = f.lines().map(|s| s.unwrap()).collect();

    let valid_irc_lines: Vec<IRCMessage> = irc_lines
        .iter()
        .filter(|&s| valid_irc(s))
        .map(|s| IRCMessage::parse(s).expect("This should be valid irc"))
        .collect();

    let mut buff = Cursor::new(vec![]);
    for line in irc_lines {
        let msg = IRCMessage::parse(&line)?;
        let msg = ServerMessage::try_from(msg)?;
        log_v0(msg, &mut buff).await;
    }

    buff.set_position(0);

    let output_lines: Vec<IRCMessage> = buff
        .lines()
        .map(|s| IRCMessage::parse(&s.unwrap()).expect("This should be valid irc"))
        .collect();

    assert_eq!(output_lines.len(), valid_irc_lines.len());
    for (output, expected) in output_lines.iter().zip(valid_irc_lines.iter()) {
        assert_eq!(output, expected);
    }

    Ok(())
}
