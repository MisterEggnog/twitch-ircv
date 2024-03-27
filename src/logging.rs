use std::io::prelude::*;

/// Log messages in IRC format
///
/// Just log all messages except ping/pong messages in IRC format.
async fn log_v0<W: Write>(message: ServerMessage, out: &mut W) {}

#[tokio::test]
async fn log_v0_tests() {}
