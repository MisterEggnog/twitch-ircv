use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

/// This was created with a lot of trial & error, mainly the tags
pub const PRIVMSG_EXAMPLE: &str = "@room-id=910;user-id=8;display-name=7;badge-info=;badges=;color=;emotes=;tmi-sent-ts=666;id=7 :bread!bread!bread@bread.tmi.twitch.tv PRIVMSG #bread :bread bread bread";

/// Generate PrivmsgMessage from PRIVMSG_EXAMPLE
///
/// This is for testing purposes
pub fn make_privmsg_example() -> twitch_irc::message::PrivmsgMessage {
    use twitch_irc::message::IRCMessage;
    IRCMessage::parse(PRIVMSG_EXAMPLE)
        .expect("Preset irc message")
        .try_into()
        .expect("This is custom designed to parse")
}

pub const PONG_MSG_EXAMPLE: &str = ":tmi.twitch.tv PONG tmi.twitch.tv tmi.twitch.tv";

#[derive(Clone)]
pub struct WriteLockBuf(Arc<Mutex<Vec<u8>>>);
impl WriteLockBuf {
    pub fn new() -> Self {
        WriteLockBuf(Arc::new(Mutex::new(vec![])))
    }

    pub fn get_data(&self) -> String {
        String::from(std::str::from_utf8(&self.0.lock().unwrap()).unwrap())
    }
}

impl Write for WriteLockBuf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

pub struct WriteIoError(pub io::ErrorKind);

impl Write for WriteIoError {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(From::from(self.0))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
