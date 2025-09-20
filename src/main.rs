mod args;
mod badges;
mod logging;
mod pretty_print;
mod setup;

use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: args::Args = argh::from_env();
    setup::init(args, io::stdin(), io::stdout()).await
}
