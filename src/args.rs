use argh::FromArgs;

/// Pretty print the live chat of a twitch channel.
/// Also offers support for logging (most) of the irc messages posted in chat.
///
/// Note:
/// * Color can be prohibited with the enviromental variable NO_COLOR.
/// * Color can be forced using the enviromental variable CLICOLOR_FORCE.
#[derive(FromArgs)]
pub struct Args {
    #[argh(positional)]
    pub channel_name: String,

    /// file to write irc log to.
    #[argh(option, short = 'o')]
    pub log_file: Option<String>,
}
