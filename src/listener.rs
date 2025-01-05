use tokio::time::Duration;

use actix::prelude::*;
use actix_rt::time::sleep;

use tracing::{info, instrument, Instrument, Span};

use crate::filter::FilterService;
use crate::model::Key;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct KeyDetected(pub Key);

pub struct KeybindListenerService {
  filter_addr: Addr<FilterService>,
}

impl KeybindListenerService {
  pub fn new(filter_addr: Addr<FilterService>) -> Self {
    Self { filter_addr }
  }
}

impl Actor for KeybindListenerService {
  type Context = Context<Self>;

  #[instrument(name = "LISTENER", skip(self, ctx))]
  fn started(&mut self, ctx: &mut Self::Context) {
    info!("listening for keypresses...");
    let filter_addr = self.filter_addr.clone();
    async move {
      let mut prev_key = Key::default();
      loop {
        let key = Key::from_async_key_state().await;
        if key != Key::default() && key != prev_key {
          prev_key = key;
          filter_addr.do_send(KeyDetected(key));
        }
        sleep(Duration::from_millis(45)).await;
      }
    }
    .instrument(Span::current())
    .into_actor(self)
    .spawn(ctx);
  }
}
