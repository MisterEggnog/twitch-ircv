use std::error::Error;
use twitch_irc::message::ServerMessage;

mod common;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    common::main_prime(|message| {
        if let ServerMessage::Privmsg(message) = message {
            println!("{} {:?}", message.sender.name, message.badges);
        }
    })
    .await
}
