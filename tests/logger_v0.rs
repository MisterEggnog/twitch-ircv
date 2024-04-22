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

    let valid_irc_lines: Vec<&str> = irc_lines
        .iter()
        .filter(|&s| valid_irc(s))
        .map(|s| s.as_str())
        .collect();

    let mut buff = Cursor::new(vec![]);
    for line in irc_lines {
        let msg = IRCMessage::parse(&line)?;
        let msg = ServerMessage::try_from(msg)?;
        log_v0(msg, &mut buff).await;
    }

    Ok(())
}
