#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::str;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use tokio_util::sync::CancellationToken;

use tracing::{info, info_span, warn, Instrument, Span};
use tracing_subscriber::filter::LevelFilter;

use dispatcher::filter::HotkeyFilter;
use dispatcher::keymap::Keymap;
use dispatcher::listener::KeybindListener;
use dispatcher::monitor::ConfigMonitor;
use dispatcher::runner::ScriptRunner;
use dispatcher::{Application, Context, Service};

struct Dispatcher {}

impl Application for Dispatcher {
  type Context = Context;
}

// current_thread or multi_thread
#[tokio::main(flavor = "current_thread")]
async fn main() {
  let logfile = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open("dispatch.log")
    .expect("open log file");

  tracing_subscriber::fmt()
    .with_max_level(LevelFilter::DEBUG)
    .with_target(false)
    .with_thread_ids(true)
    .with_ansi(false) // no escape sequences when using writer
    .with_writer(logfile)
    .init();

  log_panics::init();

  let span = info_span!("CORE");
  let _entered = span.enter();

  let services: Vec<Box<dyn Service<Context = Context> + Send + 'static>> = {
    let (tx_key, rx_key) = mpsc::unbounded_channel();
    let (tx_script, rx_script) = mpsc::unbounded_channel();
    vec![
      Box::new(KeybindListener { tx_key }),
      Box::new(ConfigMonitor {}),
      Box::new(HotkeyFilter { rx_key, tx_script }),
      Box::new(ScriptRunner { rx_script }),
    ]
  };
  let context = Context {
    token: CancellationToken::new(),
    table: Arc::new(RwLock::new(Keymap(HashMap::new()))),
  };

  // socket to halt the application on receive of b"shutdown" bytes
  let token = context.token.clone();
  let listener = TcpListener::bind("127.0.0.1:3599")
    .await
    .expect("open port 3599");

  let tcp_server = tokio::spawn(
    async move {
      use tokio::io::AsyncReadExt;
      loop {
        tokio::select! {
          Ok((mut stream, _)) = listener.accept() => {
            let mut buf = [0; 1024];
            let n = stream
              .read(&mut buf)
              .await
              .expect("read from stream into buffer");
            if let Ok(s) = str::from_utf8(&buf[0..n]) {
              let trimmed = s.trim();
              info!("received data from TCP port: {trimmed}");
              if trimmed == "shutdown" {
                warn!("shutdown received");
                warn!("STOPPING");
                token.cancel();
              }
            }
          },
          _ = token.cancelled() => break,
        }
      }
    }
    .instrument(Span::current()),
  );

  drop(_entered);

  let dispatcher = Dispatcher {};
  info!("dispatcher initialized...");
  dispatcher.invoke_all(context, services).await;
  info!("dispatcher terminated...\n\n");

  tcp_server.await.expect("listener task failed");
}
