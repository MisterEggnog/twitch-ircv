use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

fn valid_irc(s: &str) -> bool {
    s.contains("PRIVMSG")
        || s.contains("USERNOTICE")
        || s.contains("CLEARCHAT")
        || s.contains("CLEARMSG")
}

#[test]
fn test_no_ping() -> io::Result<()> {
    let f = File::open("tests/irc_data_no_ping")?;
    let f = BufReader::new(f);
    let irc_lines: Vec<String> = f.lines().map(|s| s.unwrap()).collect();

    let valid_irc_lines: Vec<&str> = irc_lines
        .iter()
        .filter(|&s| valid_irc(s))
        .map(|s| s.as_str())
        .collect();

    Ok(())
}
