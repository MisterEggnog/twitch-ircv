use twitch_irc::message::ServerMessage;

fn setup_irc_client<F>(channel: &str, message_dest: F)
where
    F: Fn(ServerMessage),
{
}
