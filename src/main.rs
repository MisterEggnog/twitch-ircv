mod args;
mod badges;
mod pretty_print;
mod logging;
mod setup;

#[tokio::main]
async fn main() {
    let args: args::Args = argh::from_env();
    setup::init(args).await;
}
