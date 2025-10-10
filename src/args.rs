use clap::Parser;
use std::path::PathBuf;

/// Pretty print the live chat of a twitch channel
#[derive(Default, Parser)]
#[command(long_about = concat!(
"Pretty print twitch chat\n",
"\n",
"Also offers support for logging (most) of the irc messages posted in chat.\n",
"\n",
"Warning:\n",
"  print-raw-irc does not give consistent output, you cannot feed the output\n",
"  of this into itself using `from-stdin` & have the same result.\n",
"\n",
"Enviromental Variables:\n",
"  • NO_COLOR: Prohibits color output.\n",
"  • CLICOLOR_FORCE: Forces color output.",
))]
pub struct Args {
    pub channel_name: String,

    /// file to write irc log to.
    #[arg(short = 'o', long)]
    pub log_file: Option<PathBuf>,

    /// append to log file, not overwrite it.
    #[arg(short = 'a', long)]
    pub append: bool,

    /// don't connect to a twitch irc channel, read raw irc from stdin.
    #[arg(long)]
    pub from_stdin: bool,

    /// output the raw irc
    #[arg(long)]
    pub print_raw_irc: bool,
}
