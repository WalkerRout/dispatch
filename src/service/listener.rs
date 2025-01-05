use tokio::time::Duration;

use actix::prelude::*;
use actix_rt::time::sleep;

use tracing::{info, instrument, Instrument, Span};

use crate::model::key::Key;
use crate::model::message::KeyDetected;

/// A listener detects keypresses, and sends them to some sink
pub struct Listener {
  filter_rec: Recipient<KeyDetected>,
}

impl Listener {
  pub fn new(filter_rec: Recipient<KeyDetected>) -> Self {
    Self { filter_rec }
  }
}

impl Actor for Listener {
  type Context = Context<Self>;

  #[instrument(name = "LISTENER", skip(self, ctx))]
  fn started(&mut self, ctx: &mut Self::Context) {
    info!("listening for keypresses...");
    let filter_rec = self.filter_rec.clone();
    async move {
      let mut prev_key = Key::default();
      loop {
        let key = Key::from_async_key_state().await;
        if key != Key::default() && key != prev_key {
          prev_key = key;
          filter_rec.do_send(KeyDetected(key));
        }
        sleep(Duration::from_millis(45)).await;
      }
    }
    .instrument(Span::current())
    .into_actor(self)
    .spawn(ctx);
  }
}
