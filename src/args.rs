use argh::FromArgs;

/// TODO
#[derive(FromArgs)]
pub struct Args {
	#[argh(positional)]
	channel_name: String,

	/// file to write irc log to.
	#[argh(option, short = 'o')]
	log_file: Option<String>,
}
