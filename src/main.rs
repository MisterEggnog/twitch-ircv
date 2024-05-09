use std::io::stdout;

mod args;
mod badges;
mod chat_logger;
mod logging;
mod setup;
use chat_logger::*;

#[tokio::main]
async fn main() {
    let args: args::Args = argh::from_env();
    let channel = args.channel_name;

    let startup_time = chrono::Utc::now();
    println!("Logging started at {}", startup_time);

    let mut stdout = stdout();

    let (mut incoming_messages, client) = setup::build_irc_client();
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            if !message_handler(message, startup_time, &mut stdout)
                .await
                .expect("Failed to write message")
            {
                break;
            }
        }
    });

    client.join(channel).unwrap();

    join_handle.await.unwrap();
}
