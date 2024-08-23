#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::sync::Arc;

use tokio::signal;
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

  let dispatcher = Dispatcher {};
  info!("dispatcher initialized...");
  #[rustfmt::skip]
  tokio::join!(
    dispatcher.invoke_all(context.clone(), services), 
    async {
      signal::ctrl_c().await.expect("listen to ctrl_c");
      info!("ctrl_c signal received");
      info!("dispatcher stopping...");
      context.token.cancel();
    }
  );
  info!("dispatcher terminated...\n\n");
}
