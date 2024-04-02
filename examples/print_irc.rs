use twitch_irc::message::AsRawIRC;

mod common;

#[tokio::main]
async fn main() {
    common::for_each_message(|message| {
        println!("{}", message.as_raw_irc());
    })
    .await
}
