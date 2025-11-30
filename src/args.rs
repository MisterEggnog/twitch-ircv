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
"Environmental Variables:\n",
"  • NO_COLOR: Prohibits color output.\n",
"  • CLICOLOR_FORCE: Forces color output.\n",
"  • TWITCH_IRCV_START_TIME={milliseconds}: Change default start time to the\n",
"    passed value."
), version, after_long_help = concat!(
"Repository: ", env!("CARGO_PKG_REPOSITORY"), "\n",
"\n",
"This program comes with ABSOLUTELY NO WARRANTY.\n",
"This is free software, and you are welcome to redistribute it under certain\n",
"conditions. For more information visit:\n",
"https://www.gnu.org/licenses/gpl-3.0.html\n"
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
