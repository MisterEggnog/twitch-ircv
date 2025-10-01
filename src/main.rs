mod args;
mod badges;
mod logging;
mod pretty_print;
mod setup;

use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    //let args: args::Args = argh::from_env();
    let args = args::Args::parse();
    setup::init(args, io::stdin(), io::stdout()).await
}
