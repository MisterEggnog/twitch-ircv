use std::io;
use std::io::prelude::*;
use twitch_irc::message::IRCParseError;

use twitch_ircv::args::Args;
use twitch_ircv::setup::init;

#[tokio::test]
async fn stdin_parsing_error_does_not_panic() -> io::Result<()> {
    let args = Args {
        from_stdin: true,
        ..Default::default()
    };
    let out = io::empty();

    let mut input = io::Cursor::new(vec![]);
    writeln!(input, "=waaa")?;
    writeln!(input, "=beans")?;
    input.rewind()?;

    let result = init(args, input, out)
        .await
        .expect_err("Input is designed to not be valid IRC");

    assert!(result.downcast::<IRCParseError>().is_ok());

    Ok(())
}
