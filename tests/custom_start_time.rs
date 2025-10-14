use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;

// Parse's datetime from the example produced by
// date +%s.%N
// To do this run
// TWITCH_IRCV_START_TIME={seconds}.{nanoseconds}
// These are seconds since UNIX timestamp
//
// This is probably parsed using DateTime::from_timestamp
#[test]
fn custom_datetime_env_variable() -> io::Result<()> {
    let input_file = "tests/irc_data_no_ping";
    let input = File::open(&input_file)?;

    let result = Command::new(env!("CARGO_BIN_EXE_twitch-ircv"))
        .args(["notachannel", "--from-stdin", "--print-raw-irc"])
        .stdin(input)
        // TODO pass TWITCH_IRCV_START_TIME env variable
        .output()?;

    let data: Vec<String> = result
        .stdout
        .lines()
        .map(|l| l.unwrap())
        .map(|l| l.trim().to_string())
        .collect();
    let err_str = String::from_utf8(result.stderr).unwrap();

    assert_eq!(result.status.code(), Some(0), "{}", err_str);
    // TODO specific tests

    Ok(())
}
