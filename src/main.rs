mod args;
mod badges;
mod chat_logger;
mod logging;
mod setup;

#[tokio::main]
async fn main() {
    let args: args::Args = argh::from_env();
    let channel = args.channel_name;

    setup::setup_irc_client(channel, |_| {}).await;
}
