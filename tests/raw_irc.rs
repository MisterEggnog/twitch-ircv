use std::fs::read_to_string;
use std::io;

use twitch_ircv::args::Args;
use twitch_ircv::setup::init;

#[tokio::test]
async fn raw_irc_mimics_input() -> io::Result<()> {
    let irc_data = read_to_string("tests/irc_data_no_ping")?;
    let args = Args {
        from_stdin: true,
        print_raw_irc: true,
        ..Default::default()
    };
    let mut output: &'static Vec<u8> = vec![];

    init(args, irc_data.clone().as_bytes(), *output)
        .await
        .expect("Bubbling up errors should count as handling them!");

    let output = String::from_utf8(*output).expect("vec written to should be utf8");
    assert_eq!(irc_data, output);

    Ok(())
}
