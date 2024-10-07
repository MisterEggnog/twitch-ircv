use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

use crate::args::Args;
use crate::logging::log_v0;
use crate::pretty_print::message_handler;

pub type TwitchClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;

pub async fn init<W, R>(args: Args, stdin: R, stdout: W)
where
    W: Write + Send + 'static,
    R: Read + Send + 'static,
{
    if args.from_stdin {
        let (handle, recv) = filein_channel_task_create(stdin);
        let (handle_res, _) = tokio::join!(handle, init_with_input(args, recv, stdout));
        handle_res.unwrap();
    } else {
        let (incoming_messages, client) = build_irc_client();
        client.join(args.channel_name.clone()).unwrap();
        init_with_input(args, incoming_messages, stdout).await;
    }
}

async fn init_with_input<W>(
    args: Args,
    incoming_messages: UnboundedReceiver<ServerMessage>,
    stdout: W,
) where
    W: Write + Send + 'static,
{
    if args.log_file.is_some() {
        let file = open_log_file(&args).unwrap();
        let mut file = io::BufWriter::new(file);

        let (handle, rx1, mut rx2) = receiver_splitter(incoming_messages);
        let fancy_task = setup_fancy_output(rx1, stdout);
        let log_task = tokio::spawn(async move {
            while let Some(message) = rx2.recv().await {
                log_v0(message, &mut file).await;
            }
        });
        let (task1, task2, task3) = tokio::join!(handle, fancy_task, log_task);
        task1.unwrap();
        task2.unwrap();
        task3.unwrap();
    } else {
        let join_handle = setup_fancy_output(incoming_messages, stdout);
        join_handle.await.unwrap();
    }
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
    let stdin_read_task = tokio::spawn(async move {
        let input = io::BufReader::new(input);
        for msg in filein_to_smsg(input) {
            tx.send(msg.unwrap()).unwrap();
        }
    });
    (stdin_read_task, rx)
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
            if tx1.send(message.clone()).is_err() || tx2.send(message).is_err() {
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

pub fn setup_fancy_output<W: Write + Send + 'static>(
    mut incoming: UnboundedReceiver<ServerMessage>,
    stdout: W,
) -> JoinHandle<()> {
    let startup_time = chrono::Utc::now();
    println!("Logging started at {}", startup_time);

    tokio::spawn(async move {
        let mut stdout = stdout;
        while let Some(message) = incoming.recv().await {
            if !message_handler(message, startup_time, &mut stdout)
                .await
                .expect("Failed to write message")
            {
                break;
            }
        }
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
async fn read_from_stdin() {
    use std::sync::{Arc, Mutex};
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

    struct WriteLockBuf(Arc<Mutex<Vec<u8>>>);
    impl Write for WriteLockBuf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }
    let output = Arc::new(Mutex::new(vec![]));

    let send_output = WriteLockBuf(Arc::clone(&output));
    let test_input = io::Cursor::new(test_input);
    init(test_args, test_input, send_output).await;

    let output = { String::from(std::str::from_utf8(&output.lock().unwrap()).unwrap()) };

    assert!(
        output.contains(expected_substr),
        "`{}` does not contain `{}`",
        output,
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
