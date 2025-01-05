#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;

use actix::prelude::*;

use tokio_util::sync::CancellationToken;

use tracing::info;
use tracing_subscriber::filter::LevelFilter;

use dispatch::config::ConfigService;
use dispatch::filter::FilterService;
use dispatch::listener::KeybindListenerService;
use dispatch::monitor::MonitorService;
use dispatch::runner::RunnerService;
use dispatch::server::ServerService;

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
  let logfile = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open("dispatch.log")
    .expect("open log file");

  tracing_subscriber::fmt()
    .with_max_level(LevelFilter::INFO)
    .with_target(false)
    .with_thread_ids(true)
    .with_ansi(false)
    .with_writer(logfile)
    .init();

  log_panics::init();

  let cancel_token = CancellationToken::new();
  let _server = ServerService::new(cancel_token.clone()).start();

  let config = ConfigService::new().start();
  let _monitor = MonitorService::new(config.clone()).start();

  let runner = RunnerService::new().start();
  let filter = FilterService::new(config.clone(), runner.clone()).start();
  let _key_listener = KeybindListenerService::new(filter.clone()).start();

  info!("all services started...");
  cancel_token.cancelled().await;
  info!("shutting down...\n");

  // explicitly shutdown current system
  System::current().stop();

  Ok(())
}
