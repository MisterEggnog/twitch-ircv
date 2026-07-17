use chrono::prelude::*;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::task::JoinHandle;
use twitch_irc::TwitchIRCClient;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::{ClientConfig, SecureTCPTransport};

use crate::args::Args;
use crate::logging::log_v0;
use crate::pretty_print::message_handler;

pub type TwitchClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;

pub async fn init<W, R>(args: Args, stdin: R, stdout: W) -> io::Result<()>
where
    W: Write + Send + 'static,
    R: Read + Send + 'static,
{
    let res = init_no_error_handling(args, stdin, stdout).await;
    if let Err(e) = res {
        if e.kind() == io::ErrorKind::BrokenPipe {
            Ok(())
        } else {
            Err(e)
        }
    } else {
        Ok(())
    }
}

pub async fn init_no_error_handling<W, R>(args: Args, stdin: R, stdout: W) -> io::Result<()>
where
    W: Write + Send + 'static,
    R: Read + Send + 'static,
{
    if args.from_stdin {
        let (handle, recv) = filein_channel_task_create(stdin);
        let (handle_res, stdout_result) = tokio::join!(handle, init_with_input(args, recv, stdout));
        handle_res.unwrap();
        stdout_result
    } else {
        let (incoming_messages, client) = build_irc_client();

        client.join(args.channel_name.clone()).unwrap();

        init_with_input(args, incoming_messages, stdout).await
    }
}

async fn init_with_input<W>(
    args: Args,
    incoming_messages: UnboundedReceiver<ServerMessage>,
    stdout: W,
) -> io::Result<()>
where
    W: Write + Send + 'static,
{
    if args.log_file.is_some() {
        let file = open_log_file(&args).unwrap();
        let file = io::BufWriter::new(file);

        write_with_log_writer(incoming_messages, stdout, file).await
    } else {
        let join_handle = setup_output(incoming_messages, &args, stdout);
        join_handle.await.unwrap()
    }
}

async fn write_with_log_writer<W1, W2>(
    incoming_messages: UnboundedReceiver<ServerMessage>,
    stdout: W1,
    mut log: W2,
) -> io::Result<()>
where
    W1: Write + Send + 'static,
    W2: Write + Send + 'static,
{
    let (handle, rx1, mut rx2) = receiver_splitter(incoming_messages);
    let stdout_task = setup_fancy_output(rx1, stdout);
    let log_task = tokio::spawn(async move {
        while let Some(message) = rx2.recv().await {
            log_v0(message, &mut log).await?;
        }
        io::Result::Ok(())
    });
    let (task1, task2, task3) = tokio::join!(handle, log_task, stdout_task);
    task1.unwrap();
    task2.unwrap()?;
    task3.unwrap()
}

fn open_log_file(args: &Args) -> io::Result<File> {
    let log_file = args.log_file.clone().unwrap();
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(args.append)
        .open(log_file)
}

fn filein_to_smsg<R: BufRead>(input: R) -> impl Iterator<Item = io::Result<ServerMessage>> {
    use twitch_irc::message::IRCMessage;
    input.lines().map(|l| {
        l.map(|raw| {
            let msg = IRCMessage::parse(raw.as_ref()).unwrap();
            ServerMessage::try_from(msg).unwrap()
        })
    })
}

fn filein_channel_task_create<R: Read + Send + 'static>(
    input: R,
) -> (JoinHandle<()>, UnboundedReceiver<ServerMessage>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let stdin_read_task = tokio::task::spawn_blocking(move || {
        let input = io::BufReader::new(input);
        for msg in filein_to_smsg(input) {
            if tx.send(msg.expect("Failed to parse irc message")).is_err() {
                break;
            }
        }
    });
    (stdin_read_task, rx)
}

async fn close_drain_receiver<T>(tx: mpsc::UnboundedSender<T>, mut rx: UnboundedReceiver<T>)
where
    T: std::marker::Send + 'static,
{
    rx.close();
    while let Some(message) = rx.recv().await {
        if tx.send(message).is_err() {
            return;
        }
    }
}

fn receiver_splitter<T>(
    mut incoming: UnboundedReceiver<T>,
) -> (JoinHandle<()>, UnboundedReceiver<T>, UnboundedReceiver<T>)
where
    T: Clone + std::marker::Send + 'static,
{
    let (tx1, rx1) = mpsc::unbounded_channel();
    let (tx2, rx2) = mpsc::unbounded_channel();
    let handle = tokio::spawn(async move {
        while let Some(message) = incoming.recv().await {
            let res1 = tx1.send(message.clone());
            let res2 = tx2.send(message);
            if res1.is_err() && res2.is_err() {
                return;
            } else if res1.is_err() && res2.is_ok() {
                close_drain_receiver(tx2, incoming).await;
                return;
            } else if res1.is_ok() && res2.is_err() {
                close_drain_receiver(tx1, incoming).await;
                return;
            }
        }
    });
    (handle, rx1, rx2)
}

/// Simplified version of TwitchIRCClient::new with default config
pub fn build_irc_client() -> (UnboundedReceiver<ServerMessage>, TwitchClient) {
    let config = ClientConfig::default();
    TwitchClient::new(config)
}

pub fn setup_output<W: Write + Send + 'static>(
    mut incoming: UnboundedReceiver<ServerMessage>,
    args: &Args,
    stdout: W,
) -> JoinHandle<io::Result<()>> {
    use twitch_irc::message::AsRawIRC;
    if args.print_raw_irc {
        tokio::spawn(async move {
            let mut stdout = stdout;
            while let Some(message) = incoming.recv().await {
                writeln!(stdout, "{}", message.as_raw_irc())?;
            }
            Ok(())
        })
    } else {
        setup_fancy_output(incoming, stdout)
    }
}

fn arg_str_time_parse(timestr: Result<String, env::VarError>) -> Option<DateTime<Utc>> {
    let message_str = timestr.ok()?;
    let milli = message_str.parse().ok()?;
    DateTime::from_timestamp_millis(milli)
}

pub fn setup_fancy_output<W: Write + Send + 'static>(
    mut incoming: UnboundedReceiver<ServerMessage>,
    stdout: W,
) -> JoinHandle<io::Result<()>> {
    let startup_time =
        arg_str_time_parse(env::var("TWITCH_IRCV_START_TIME")).unwrap_or_else(Utc::now);
    println!("Logging started at {}", startup_time);

    tokio::spawn(async move {
        let mut stdout = stdout;
        while let Some(message) = incoming.recv().await {
            message_handler(message, startup_time, &mut stdout).await?;
        }
        Ok(())
    })
}

/// This was created with a lot of trial & error, mainly the tags
#[allow(dead_code)]
pub const PRIVMSG_EXAMPLE: &str = "@room-id=910;user-id=8;display-name=7;badge-info=;badges=;color=;emotes=;tmi-sent-ts=666;id=7 :bread!bread!bread@bread.tmi.twitch.tv PRIVMSG #bread :bread bread bread";

/// Generate PrivmsgMessage from PRIVMSG_EXAMPLE
///
/// This is for testing purposes
#[allow(dead_code)]
pub fn make_privmsg_example() -> twitch_irc::message::PrivmsgMessage {
    use twitch_irc::message::IRCMessage;
    IRCMessage::parse(PRIVMSG_EXAMPLE)
        .expect("Preset irc message")
        .try_into()
        .expect("This is custom designed to parse")
}

#[allow(dead_code)]
pub const PONG_MSG_EXAMPLE: &str = ":tmi.twitch.tv PONG tmi.twitch.tv tmi.twitch.tv";

#[allow(unused)]
pub struct WriteIoError(pub io::ErrorKind);

impl Write for WriteIoError {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(From::from(self.0))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::WriteLockBuf;

    #[tokio::test]
    async fn write_raw_irc_matches_input() {
        use tokio::sync::mpsc::unbounded_channel;
        use twitch_irc::message::{AsRawIRC, IRCMessage, ServerMessage};

        let example = IRCMessage::parse(PRIVMSG_EXAMPLE).unwrap();
        let example = ServerMessage::try_from(example).unwrap();
        let args = Args {
            print_raw_irc: true,
            ..Default::default()
        };
        let privmsg_example = format!("{}\n", example.as_raw_irc());
        let fake_stdout = WriteLockBuf::new();

        let (input, output) = unbounded_channel();
        input.send(example).unwrap();
        drop(input);
        let _ = setup_output(output, &args, fake_stdout.clone())
            .await
            .unwrap();

        // This program will change the order of the irc message tags when
        // building the `source` message, so I need to do this.
        let output_data = fake_stdout.get_data();
        assert_eq!(privmsg_example, output_data);
    }

    #[test]
    fn append_switch_works() -> std::io::Result<()> {
        use std::fs::read_to_string;
        use std::io::Write;
        use tempfile::NamedTempFile;
        let mut path = NamedTempFile::new().expect("Could not get temp path");

        let log_file = Some(path.as_ref().to_path_buf());
        let append = true;
        let test_args = Args {
            log_file,
            append,
            ..Default::default()
        };

        writeln!(path, "Bagginses")?;

        let mut outfs = open_log_file(&test_args)?;
        writeln!(outfs, "I am full of spaghetti.")?;

        drop(outfs);

        let file_contents = read_to_string(path.as_ref())?;
        let expected = "Bagginses\nI am full of spaghetti.\n";
        assert_eq!(file_contents, expected);

        Ok(())
    }

    #[test]
    fn arg_str_time_parse_parses_valid_str() {
        let milliseconds = 1761108680812;
        let datetime_str = format!("{}", milliseconds);
        let datetime = DateTime::from_timestamp_millis(milliseconds).unwrap();
        let result = arg_str_time_parse(Ok(datetime_str)).expect("failed to parse arg str");
        assert_eq!(
            datetime, result,
            "Expected: {}, result: {}",
            datetime, result
        );
    }

    #[test]
    fn env_var_get_time_returns_none_on_bad_cases() {
        use std::env::VarError;
        let result_not_present = arg_str_time_parse(Err(VarError::NotPresent));
        assert!(result_not_present.is_none());

        let result_from_garbage = arg_str_time_parse(Ok(String::from("eeeyiay")));
        assert!(result_from_garbage.is_none());
    }

    #[test]
    fn open_log_file_opens_write_by_default() -> io::Result<()> {
        use std::fs::read_to_string;
        use std::io::Write;
        use tempfile::NamedTempFile;
        let mut path = NamedTempFile::new().expect("Could not get temp path");
        writeln!(path, "Bagginses")?;

        let log_file = Some(path.as_ref().to_path_buf());
        let test_args = Args {
            log_file,
            append: false,
            ..Default::default()
        };
        let mut outfs = open_log_file(&test_args)?;
        writeln!(outfs, "I am full of spaghetti.")?;
        drop(outfs);

        let file_contents = read_to_string(path.as_ref())?;
        assert_eq!("I am full of spaghetti.\n", file_contents);

        Ok(())
    }

    #[tokio::test]
    async fn log_writer_cleanly_handles_errors() {
        use tokio::sync::mpsc::unbounded_channel;
        let (tx, messages) = unbounded_channel();

        // For some reason if this is not a different thread this test will run
        // indefinitely.
        let messenger_task = tokio::task::spawn_blocking(move || {
            use twitch_irc::message::IRCMessage;
            let irc_message = IRCMessage::parse(PRIVMSG_EXAMPLE).expect("custom built irc msg");
            let irc_message = ServerMessage::try_from(irc_message).expect("This is a privmsg");
            for _ in 0..10 {
                tx.send(irc_message.clone()).expect("This is unbounded");
            }
        });

        let stdout = io::empty();
        let log = WriteIoError(io::ErrorKind::StorageFull);
        let result = write_with_log_writer(messages, stdout, log).await;

        let error = result.expect_err("writer should fail with StorageFull");
        assert_eq!(error.kind(), io::ErrorKind::StorageFull);

        messenger_task
            .await
            .expect("this task should produce messages until there is no one left to here them.");
    }

    #[tokio::test]
    async fn read_from_stdin() {
        use twitch_irc::message::{AsRawIRC, IRCMessage, ServerMessage};
        let test_args = Args {
            channel_name: String::from("&"),
            from_stdin: true,
            ..Default::default()
        };

        let msg = IRCMessage::parse(PRIVMSG_EXAMPLE).unwrap();
        let msg = ServerMessage::try_from(msg).unwrap();

        let pong_msg = IRCMessage::parse(PONG_MSG_EXAMPLE).unwrap();
        let pong_msg = ServerMessage::try_from(pong_msg).unwrap();

        let expected_substr = "7: bread bread bread";

        let mut test_input = vec![];
        writeln!(test_input, "{}", pong_msg.as_raw_irc()).unwrap();
        writeln!(test_input, "{}", msg.as_raw_irc()).unwrap();
        writeln!(test_input, "{}", pong_msg.as_raw_irc()).unwrap();

        let output = WriteLockBuf::new();

        let test_input = io::Cursor::new(test_input);
        let _ = init(test_args, test_input, output.clone()).await;

        let output_data = output.get_data();

        assert!(
            output_data.contains(expected_substr),
            "`{}` does not contain `{}`",
            output_data,
            expected_substr
        );
    }

    #[test]
    fn test_text_to_server_message() {
        use twitch_irc::message::IRCMessage;
        let msg = IRCMessage::parse(PRIVMSG_EXAMPLE).unwrap();
        let msg = ServerMessage::try_from(msg).unwrap();

        let pong_msg = IRCMessage::parse(PONG_MSG_EXAMPLE).unwrap();
        let pong_msg = ServerMessage::try_from(pong_msg).unwrap();

        let mut test_input = vec![];
        writeln!(test_input, "{}", PRIVMSG_EXAMPLE).unwrap();
        writeln!(test_input, "{}", PONG_MSG_EXAMPLE).unwrap();
        writeln!(test_input, "{}", PRIVMSG_EXAMPLE).unwrap();
        let test_input = io::Cursor::new(test_input);

        // I understand why ServerMessage doesn't impl PartialEq but it makes
        // testing difficult.
        let expected: Vec<_> = [msg.clone(), pong_msg, msg].into();
        let result: Vec<_> = filein_to_smsg(test_input).map(|s| s.unwrap()).collect();
        assert_eq!(expected.len(), result.len());
        for (res, exp) in expected.into_iter().zip(result) {
            assert_eq!(res.source(), exp.source());
        }
    }

    #[tokio::test]
    async fn create_stdin_task() {
        use twitch_irc::message::IRCMessage;
        let irc_msg = IRCMessage::parse(PRIVMSG_EXAMPLE).unwrap();

        let mut input = vec![];
        writeln!(input, "{}", PRIVMSG_EXAMPLE).unwrap();
        writeln!(input, "{}", PRIVMSG_EXAMPLE).unwrap();
        let input = io::Cursor::new(input);

        let (handle, mut incoming) = filein_channel_task_create(input);
        let first = incoming.recv().await.unwrap();
        assert_eq!(first.source(), &irc_msg);

        let second = incoming.recv().await.unwrap();
        assert_eq!(second.source(), &irc_msg);
        assert!(incoming.recv().await.is_none());

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn receiver_splitter_is_balanced() {
        let (tx, rx) = mpsc::unbounded_channel();
        let (handle, mut out1, mut out2) = receiver_splitter(rx);
        let test_msg = "Hewwo, I am a string";

        tx.send(test_msg).unwrap();
        let res1 = out1.recv().await.unwrap();
        let res2 = out2.recv().await.unwrap();

        assert_eq!(test_msg, res1);
        assert_eq!(test_msg, res2);

        drop(tx);
        handle.await.unwrap();
    }

    async fn receiver_splitter_drains_side(
        tx: mpsc::UnboundedSender<i32>,
        mut rx: UnboundedReceiver<i32>,
        dies: UnboundedReceiver<i32>,
    ) {
        drop(dies);
        tx.send(0).expect("Should be able to send");
        tx.send(1).expect("Should be able to send");
        assert_eq!(rx.recv().await, Some(0));
        assert_eq!(rx.recv().await, Some(1));
        assert_eq!(rx.recv().await, None);
    }

    #[tokio::test]
    async fn receiver_splitter_drains_to_remaining_channel() {
        let (tx, rx) = mpsc::unbounded_channel();
        let (handle, out1, out2) = receiver_splitter(rx);
        receiver_splitter_drains_side(tx, out1, out2).await;
        handle.await.expect("task failed");

        let (tx, rx) = mpsc::unbounded_channel();
        let (handle, out1, out2) = receiver_splitter(rx);
        receiver_splitter_drains_side(tx, out2, out1).await;
        handle.await.expect("task failed");
    }
}
