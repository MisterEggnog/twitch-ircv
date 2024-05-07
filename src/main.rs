use std::sync::mpsc::channel;

mod args;
mod badges;
mod chat_logger;
mod logging;
mod setup;

#[tokio::main]
async fn main() {
    let args: args::Args = argh::from_env();
    let channel_name = args.channel_name;

    let (sender, receiver) = channel::<()>();

    setup::setup_irc_client(channel_name, |_| {}).await;
}
