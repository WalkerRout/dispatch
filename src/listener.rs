use std::time::Duration;

use async_trait::async_trait;

use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;

use tokio_util::sync::CancellationToken;

use tracing::{error, instrument, warn};

use crate::key::Key;
use crate::{Context, Service};

pub struct KeybindListener {
  pub tx_key: UnboundedSender<Key>,
}

impl KeybindListener {
  async fn listen_for_keypresses(&mut self, token: CancellationToken) {
    let mut prev_key = Key { repr: 0 };
    loop {
      let key = Key::from_async_key_state().await;

      if key.repr != 0 && key != prev_key {
        prev_key = key;
        if let Err(e) = self.tx_key.send(key) {
          error!("failed to send across tx_key - {e}");
          warn!("STOPPING");
          token.cancel();
        }
      }

      sleep(Duration::from_millis(45)).await;
    }
  }
}

#[async_trait]
impl Service for KeybindListener {
  type Context = Context;
  #[instrument(name = "LISTENER", skip(self, ctx))]
  async fn invoke(&mut self, ctx: Self::Context) {
    tokio::select! {
      () = self.listen_for_keypresses(ctx.token.clone()) => (),
      () = ctx.token.cancelled() => (),
    }
    warn!("stopping gracefully");
  }
}
