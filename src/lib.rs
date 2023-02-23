use twitch_irc::message::ServerMessage;

pub async fn message_handler(message: ServerMessage) {
    match message {
        ServerMessage::Privmsg(msg) => println!("{:?}", msg),
        _ => (),
    }
}
