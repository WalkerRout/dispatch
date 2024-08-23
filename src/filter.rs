use std::sync::Arc;

use async_trait::async_trait;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

use tokio_util::sync::CancellationToken;

use tracing::{error, info, instrument, warn};

use crate::key::Key;
use crate::keymap::Keymap;
use crate::script::Script;
use crate::{Context, Service};

pub struct HotkeyFilter {
  pub rx_key: UnboundedReceiver<Key>,
  pub tx_script: UnboundedSender<Script>,
}

impl HotkeyFilter {
  async fn filter_hotkeys(&mut self, token: CancellationToken, table: Arc<RwLock<Keymap>>) {
    loop {
      match self.rx_key.recv().await {
        Some(key) => {
          let script = {
            let snapshot = table.read().await;
            match snapshot.get(&key) {
              Some(s) => s.clone(),
              None => continue,
            }
          };
          info!("received mapping from {key:?} to {}", &script);
          if let Err(e) = self.tx_script.send(script) {
            error!("failed to send across tx_script - {e}");
            warn!("STOPPING");
            token.cancel();
          }
        }
        None => {
          error!("failed to receive across rx_key");
          warn!("STOPPING");
          token.cancel();
        }
      }
    }
  }
}

#[async_trait]
impl Service for HotkeyFilter {
  type Context = Context;
  #[instrument(name = "FILTER", skip(self, ctx))]
  async fn invoke(&mut self, ctx: Self::Context) {
    tokio::select! {
      () = self.filter_hotkeys(ctx.token.clone(), Arc::clone(&ctx.table)) => {},
      () = ctx.token.cancelled() => (),
    }
    warn!("stopping gracefully");
  }
}
