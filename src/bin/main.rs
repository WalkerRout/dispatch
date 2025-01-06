#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;

use actix::prelude::*;

use tokio_util::sync::CancellationToken;

use tracing::info;
use tracing_subscriber::filter::LevelFilter;

use dispatch::service::config::Config;
use dispatch::service::filter::Filter;
use dispatch::service::listener::Listener;
use dispatch::service::monitor::Monitor;
use dispatch::service::runner::Runner;
use dispatch::service::server::Server;

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
  let _server = Server::new(cancel_token.clone()).start();

  let config = Config::new().start();
  let _monitor = Monitor::new(config.clone().into()).start();

  let runner = Runner::new().start();
  let filter = Filter::new(config.clone().into(), runner.clone().into()).start();
  let _listener = Listener::new(filter.clone().into()).start();

  info!("all services started...");

  cancel_token.cancelled().await;

  Ok(())
}
