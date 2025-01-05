use actix::prelude::*;

use tracing::instrument;

use crate::model::keymap::Keymap;
use crate::model::script::Script;
use crate::model::message::{GetScript, UpdateConfig};

/// A config is a keymap that represents a 'global truth' for which keys are valid
#[derive(Default)]
pub struct Config {
  commands: Keymap,
}

impl Config {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Actor for Config {
  type Context = Context<Self>;
}

// others can request (get) scripts from this config...
impl Handler<GetScript> for Config {
  type Result = Option<Script>;

  #[instrument(name = "CONFIG", skip(self, _ctx))]
  fn handle(&mut self, msg: GetScript, _ctx: &mut Context<Self>) -> Self::Result {
    self.commands.get(&msg.0).cloned()
  }
}

// others can also update (set) this config...
impl Handler<UpdateConfig> for Config {
  type Result = ();

  #[instrument(name = "CONFIG", skip(self, _ctx))]
  fn handle(&mut self, msg: UpdateConfig, _ctx: &mut Context<Self>) -> Self::Result {
    self.commands = msg.0;
  }
}
