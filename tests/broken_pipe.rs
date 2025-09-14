use std::fs::File;
use std::io;
use std::io::prelude::*;

use twitch_ircv::args::Args;
use twitch_ircv::setup::init;

struct PanicsBrokenPipe;

impl Write for PanicsBrokenPipe {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(From::from(io::ErrorKind::BrokenPipe))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn cleanly_exits_with_broken_pipe() -> io::Result<()> {
    let fake_stdin = File::open("tests/irc_data_no_ping")?;
    let args = Args {
        from_stdin: true,
        ..Default::default()
    };

    init(args, fake_stdin, PanicsBrokenPipe).await
}
