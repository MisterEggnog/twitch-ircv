use argh::FromArgs;

/// TODO
#[derive(FromArgs)]
pub struct Args {
	#[argh(positional)]
	pub channel_name: String,

	/// file to write irc log to.
	#[argh(option, short = 'o')]
	pub log_file: Option<String>,
}
