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

pub async fn init(args: Args) {
    let (incoming_messages, client) = build_irc_client();
    client.join(args.channel_name.clone()).unwrap();

    init_with_input(args, incoming_messages, io::stdin(), io::stdout()).await;
}

async fn init_with_input<W, R>(
    args: Args,
    incoming_messages: UnboundedReceiver<ServerMessage>,
    stdin: R,
    stdout: W,
) where
    W: Write + Send + 'static,
    R: Read + Send + 'static,
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
    //let append = args.append.unwrap_or(false);
    let log_file = args.log_file.clone().unwrap();
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(args.append)
        .open(log_file)
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
    //let mut stdout = io::stdout();

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
    let test_args = Args {
        channel_name: String::from("&"),
        from_stdin: true,
        ..Default::default()
    };
    //let (msg_dest, msg_source) = mpsc::unbounded_channel();

    struct WriteLockBuf(Arc<Mutex<Vec<u8>>>);
    impl Write for WriteLockBuf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }
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
