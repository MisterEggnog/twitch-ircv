mod args;
mod badges;
mod chat_logger;
mod logging;
mod setup;

#[tokio::main]
async fn main() {
    let args: args::Args = argh::from_env();
    let channel = args.channel_name;

    let (incoming_messages, client) = setup::build_irc_client();
    let join_handle = setup::setup_fancy_output(incoming_messages);

    client.join(channel).unwrap();

    join_handle.await.unwrap();
}
