use twitch_irc::message::ServerMessage;

pub async fn message_handler(message: ServerMessage) {
    println!("{:?}", message);
}
