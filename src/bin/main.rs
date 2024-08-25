#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use tokio_util::sync::CancellationToken;

use tracing::info;
use tracing_subscriber::filter::LevelFilter;

use dispatcher::filter::HotkeyFilter;
use dispatcher::keymap::Keymap;
use dispatcher::listener::KeybindListener;
use dispatcher::monitor::ConfigMonitor;
use dispatcher::runner::ScriptRunner;
use dispatcher::server::TcpServer;
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
    .with_ansi(false)
    .with_writer(logfile)
    .init();

  log_panics::init();

  let services: Vec<Box<dyn Service<Context = Context> + Send + 'static>> = {
    let (tx_key, rx_key) = mpsc::unbounded_channel();
    let (tx_script, rx_script) = mpsc::unbounded_channel();
    let listener = TcpListener::bind("127.0.0.1:3599")
      .await
      .expect("open port 3599");
    vec![
      Box::new(TcpServer { listener }),
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

  let dispatcher = Dispatcher {};
  info!("dispatcher initialized...");
  dispatcher.invoke_all(context, services).await;
  info!("dispatcher terminated...\n\n");
}
