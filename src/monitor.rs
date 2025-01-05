use std::fs;
use std::path::Path;

use actix::prelude::*;

use tokio::sync::mpsc::{self, Receiver};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use tracing::{error, info, instrument, warn, Instrument, Span};

use crate::config::{ConfigService, UpdateConfig};
use crate::model::parse_json;

pub enum WatcherPayload {
  Bytes(Vec<u8>),
  Error(anyhow::Error),
}

pub struct MonitorService {
  config_addr: Addr<ConfigService>,
  _watcher: Option<RecommendedWatcher>,
}

impl MonitorService {
  pub fn new(config_addr: Addr<ConfigService>) -> Self {
    Self {
      config_addr,
      _watcher: None,
    }
  }
}

impl Actor for MonitorService {
  type Context = Context<Self>;

  #[instrument(name = "MONITOR", skip(self, ctx))]
  fn started(&mut self, ctx: &mut Self::Context) {
    let path = "dispatch.json";
    let (mut file_events, watcher) = match async_watcher(path) {
      Ok(obs) => obs,
      Err(e) => panic!("{e}"),
    };
    let config_addr = self.config_addr.clone();

    self._watcher = Some(watcher);

    async move {
      loop {
        match file_events.recv().await {
          Some(WatcherPayload::Bytes(bytes)) => {
            // sent nothing, try again
            if bytes.is_empty() {
              continue;
            }
            match parse_json(&bytes[..]) {
              Ok(new_map) => {
                config_addr.do_send(UpdateConfig(new_map));
                info!("dispatch config successfully updated");
              }
              Err(e) => warn!("invalid dispatch config state saved - {e}"),
            }
          }
          Some(WatcherPayload::Error(e)) => panic!("monitor failed to track changes - {e}"),
          None => panic!("failed to receive from monitor tx"),
        }
      }
    }
    .instrument(Span::current())
    .into_actor(self)
    .spawn(ctx);
  }
}

fn async_watcher<P: AsRef<Path>>(
  path: P,
) -> Result<(Receiver<WatcherPayload>, RecommendedWatcher), anyhow::Error> {
  let (tx, rx) = mpsc::channel(4);

  let mut watcher = {
    let tx = tx.clone();
    notify::recommended_watcher(move |res| match res {
      Ok(Event {
        kind: EventKind::Modify(_),
        paths,
        ..
      }) => {
        assert_eq!(paths.len(), 1);
        // build up payload from source
        let mut bytes = Vec::new();
        match fs::read(&paths[0]) {
          Ok(extra) => bytes.extend(extra),
          Err(e) => {
            if let Err(e) = tx.try_send(WatcherPayload::Error(anyhow::Error::from(e))) {
              error!("failed to send payload from watcher - {e}");
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

  let path = path.as_ref();

  let mut initial_config = Vec::new();
  if let Ok(bytes) = fs::read(path) {
    initial_config.extend(bytes);
  }

  watcher
    .watch(path, RecursiveMode::Recursive)
    .inspect_err(|_| {
      warn!("{} not found", path.display());
    })?;

  // send initial config from first read
  if let Err(e) = tx.try_send(WatcherPayload::Bytes(initial_config)) {
    error!("failed to send initial config - {e}");
  }

  Ok((rx, watcher))
}
