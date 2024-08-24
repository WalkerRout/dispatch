use std::fs;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::RwLock;

use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, warn};

use crate::keymap::Keymap;
use crate::{Context, Service};

pub struct ConfigMonitor {}

impl ConfigMonitor {
  async fn monitor_files(&mut self, token: CancellationToken, table: Arc<RwLock<Keymap>>) {
    let paths = vec!["dispatch.json"];

    let (mut file_events, _watcher) = match async_watcher(paths) {
      Ok(obs) => obs,
      Err(e) => {
        error!("unable to debounce watch - {e}");
        warn!("STOPPING");
        token.cancel();
        return;
      }
    };

    loop {
      match file_events.recv().await {
        Some(WatcherPayload::Bytes(bytes)) => {
          // sent nothing, try again
          if bytes.is_empty() {
            continue;
          }
          match Keymap::try_from(&bytes[..]) {
            Ok(new_map) => {
              *table.write().await = new_map;
              info!("dispatch config successfully updated");
            }
            Err(e) => warn!("invalid dispatch config state saved - {e}"),
          }
        }
        Some(WatcherPayload::Error(e)) => {
          error!("monitor failed to track changes - {e}");
          warn!("STOPPING");
          token.cancel();
        }
        None => {
          error!("failed to receive from monitor tx");
          warn!("STOPPING");
          token.cancel();
        }
      }
    }
  }
}

#[async_trait]
impl Service for ConfigMonitor {
  type Context = Context;
  #[instrument(name = "CONFIG", skip(self, ctx))]
  async fn invoke(&mut self, ctx: Self::Context) {
    tokio::select! {
      () = self.monitor_files(ctx.token.clone(), Arc::clone(&ctx.table)) => {},
      () = ctx.token.cancelled() => (),
    }
    warn!("stopping gracefully");
  }
}

enum WatcherPayload {
  Bytes(Vec<u8>),
  Error(anyhow::Error),
}

fn async_watcher<P: AsRef<Path>>(
  paths: Vec<P>,
) -> Result<(Receiver<WatcherPayload>, RecommendedWatcher), anyhow::Error> {
  // can only handle single file as of now
  assert_eq!(paths.len(), 1);

  let (tx, rx) = mpsc::channel(1);

  let mut watcher = {
    let tx = tx.clone();
    notify::recommended_watcher(move |res| match res {
      Ok(Event {
        kind: EventKind::Modify(_),
        paths,
        ..
      }) => {
        // build up payload from sources
        let mut bytes = Vec::new();
        for path in paths {
          match fs::read(path) {
            Ok(extra) => bytes.extend(extra),
            Err(e) => {
              if let Err(e) = tx.try_send(WatcherPayload::Error(anyhow::Error::from(e))) {
                error!("failed to send payload from watcher - {e}");
              }
            }
          }
        }

        if let Err(e) = tx.try_send(WatcherPayload::Bytes(bytes)) {
          error!("failed to send payload from watcher - {e}");
        }
      }
      // ignore non-modify events
      Ok(_) => (),
      Err(e) => error!("failed to determine watcher payload - {e}"),
    })?
  };

  let mut initial_config = Vec::new();
  for path in paths {
    let path = path.as_ref();

    if let Ok(bytes) = fs::read(path) {
      initial_config.extend(bytes);
    }

    watcher
      .watch(path, RecursiveMode::Recursive)
      .inspect_err(|_| {
        warn!("{} not found", path.display());
      })?;
  }

  // send initial config from first read
  if let Err(e) = tx.try_send(WatcherPayload::Bytes(initial_config)) {
    error!("failed to send initial config - {e}");
  }

  Ok((rx, watcher))
}
