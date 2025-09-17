use std::fs::{read_to_string, File};
use std::io;
use std::process::Command;

#[test]
fn raw_irc_mimics_input() -> io::Result<()> {
    let input_file = "tests/irc_data_no_ping";
    let irc_data = read_to_string(&input_file)?;

    let input = File::open(&input_file)?;
    let result = Command::new(env!("CARGO_BIN_EXE_twitch-ircv"))
        .args(["notachannel", "--from-stdin", "--print-raw-irc"])
        .stdin(input)
        .output()?;
    let data = String::from_utf8(result.stdout).expect("terminal output should be utf8");
    let err_str = String::from_utf8(result.stderr).unwrap();
    assert_eq!(result.status.code(), Some(0), "{}", err_str);
    assert_eq!(irc_data, data);

    Ok(())
}
