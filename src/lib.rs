use twitch_irc::message::PrivmsgMessage;
use twitch_irc::message::ServerMessage;

pub async fn message_handler(message: ServerMessage) {
    match message {
        ServerMessage::Privmsg(msg) => print_chat_msg(msg).await,
        _ => (),
    }
}

async fn print_chat_msg(msg: PrivmsgMessage) {
    println!("{:?}", msg);
}
