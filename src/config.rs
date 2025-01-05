use actix::prelude::*;

use tracing::instrument;

use crate::model::{Key, Keymap, Script};

#[derive(Debug, Message)]
#[rtype(result = "Option<String>")]
pub struct GetScript(pub Key);

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct UpdateConfig(pub Keymap);

#[derive(Default)]
pub struct ConfigService {
  commands: Keymap,
}

impl ConfigService {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Actor for ConfigService {
  type Context = Context<Self>;
}

impl Handler<GetScript> for ConfigService {
  type Result = Option<Script>;

  #[instrument(name = "CONFIG", skip(self, _ctx))]
  fn handle(&mut self, msg: GetScript, _ctx: &mut Context<Self>) -> Self::Result {
    self.commands.get(&msg.0).cloned()
  }
}

impl Handler<UpdateConfig> for ConfigService {
  type Result = ();

  #[instrument(name = "CONFIG", skip(self, _ctx))]
  fn handle(&mut self, msg: UpdateConfig, _ctx: &mut Context<Self>) -> Self::Result {
    self.commands = msg.0;
  }
}
