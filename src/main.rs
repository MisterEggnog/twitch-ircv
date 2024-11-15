mod args;
mod badges;
mod logging;
mod pretty_print;
mod setup;

use std::io::{stdin, stdout};

#[tokio::main]
async fn main() {
    let args: args::Args = argh::from_env();
    setup::init(args, stdin(), stdout()).await;
}
