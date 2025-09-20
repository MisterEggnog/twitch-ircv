use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;

use twitch_irc::message::AsRawIRC;
use twitch_irc::message::IRCMessage;

#[test]
fn raw_irc_mimics_input() -> io::Result<()> {
    let input_file = "tests/irc_data_no_ping";
    let irc_data: Vec<String> = BufReader::new(File::open(&input_file)?)
        .lines()
        .map(|l| l.unwrap())
        .map(|l| {
            let l = l.trim();
            let irc = IRCMessage::parse(&l).expect("This was irc to begin with");
            // Program changes layout of tags during processing
            // this doesn't fix all of the issues.
            irc.as_raw_irc()
        })
        .collect();

    let input = File::open(&input_file)?;
    let result = Command::new(env!("CARGO_BIN_EXE_twitch-ircv"))
        .args(["notachannel", "--from-stdin", "--print-raw-irc"])
        .stdin(input)
        .output()?;

    let data: Vec<String> = result
        .stdout
        .lines()
        .map(|l| l.unwrap())
        .map(|l| l.trim().to_string())
        .collect();
    let err_str = String::from_utf8(result.stderr).unwrap();

    assert_eq!(result.status.code(), Some(0), "{}", err_str);
    assert_eq!(irc_data.len(), data.len(), "irc line count differs");
    for (example, result) in irc_data.iter().zip(data.iter()) {
        assert_eq!(example, result);
    }

    Ok(())
}
