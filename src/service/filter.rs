use std::future::Future;
use std::pin::Pin;

use actix::prelude::*;

use tracing::{error, instrument, warn, Instrument, Span};

use crate::model::key::Key;
// the filter can detect keys, get scripts, and send commands...
use crate::model::message::{GetScript, KeyDetected, RunCommand};

/// A filter can retrieve a script from somewhere and can send it somewhere else
pub struct Filter {
  config_rec: Recipient<GetScript>,
  runner_rec: Recipient<RunCommand>,
}

impl Filter {
  pub fn new(config_rec: Recipient<GetScript>, runner_rec: Recipient<RunCommand>) -> Self {
    Self {
      config_rec,
      runner_rec,
    }
  }

  fn process_key(&mut self, key: Key) -> Pin<Box<dyn Future<Output = ()>>> {
    let config_rec = self.config_rec.clone();
    let runner_rec = self.runner_rec.clone();
    Box::pin(async move {
      match config_rec.send(GetScript(key)).await {
        Ok(Some(cmd)) => runner_rec.do_send(RunCommand(cmd)),
        Err(e) => error!("error while querying config: {e}"),
        _ => (),
      }
    })
  }
}

impl Actor for Filter {
  type Context = Context<Self>;
}

impl Handler<KeyDetected> for Filter {
  type Result = ();

  #[instrument(name = "FILTER", skip(self, msg, ctx))]
  fn handle(&mut self, msg: KeyDetected, ctx: &mut Context<Self>) -> Self::Result {
    self
      .process_key(msg.0)
      .instrument(Span::current())
      .into_actor(self)
      .spawn(ctx);
  }
}
