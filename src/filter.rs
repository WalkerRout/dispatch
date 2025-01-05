use actix::prelude::*;

use tracing::{error, instrument, warn, Instrument, Span};

use crate::config::{ConfigService, GetScript};
use crate::listener::KeyDetected;
use crate::runner::{RunCommand, RunnerService};

pub struct FilterService {
  config_addr: Addr<ConfigService>,
  runner_addr: Addr<RunnerService>,
}

impl FilterService {
  pub fn new(config_addr: Addr<ConfigService>, runner_addr: Addr<RunnerService>) -> Self {
    Self {
      config_addr,
      runner_addr,
    }
  }
}

impl Actor for FilterService {
  type Context = Context<Self>;
}

impl Handler<KeyDetected> for FilterService {
  type Result = ();

  #[instrument(name = "FILTER", skip(self, msg, ctx))]
  fn handle(&mut self, msg: KeyDetected, ctx: &mut Context<Self>) -> Self::Result {
    let config_addr = self.config_addr.clone();
    let runner_addr = self.runner_addr.clone();
    async move {
      match config_addr.send(GetScript(msg.0)).await {
        Ok(Some(cmd)) => runner_addr.do_send(RunCommand(cmd)),
        Err(e) => error!("error while querying config: {e}"),
        _ => (),
      }
    }
    .instrument(Span::current())
    .into_actor(self)
    .spawn(ctx);
  }
}
