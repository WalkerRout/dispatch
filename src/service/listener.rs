use std::future::Future;
use std::pin::Pin;

use actix::prelude::*;

use tokio::time::{sleep, Duration};

use tracing::{info, instrument, warn, Instrument, Span};

use device_query::{DeviceQuery, DeviceState, Keycode};

use crate::model::key::Key;
use crate::model::message::{KeyDetected, KeyState};

/// A listener detects keypresses, and sends them to some sink
pub struct Listener {
  filter_rec: Recipient<KeyDetected>,
  prev_key: Option<Key>,
}

impl Listener {
  pub fn new(filter_rec: Recipient<KeyDetected>) -> Self {
    Self {
      filter_rec,
      prev_key: None,
    }
  }

  fn watch_keystate(
    &mut self,
    state_rec: Recipient<KeyState>,
  ) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
      let device_state = DeviceState::new();
      info!("starting device query loop...");
      // we dont ever really need to stop this loop since our executor can just
      // abort this task for us..
      loop {
        state_rec.do_send(KeyState(device_state.get_keys()));
        sleep(Duration::from_millis(45)).await;
      }
    })
  }
}

impl Actor for Listener {
  type Context = Context<Self>;

  #[instrument(name = "LISTENER", skip(self, ctx))]
  fn started(&mut self, ctx: &mut Self::Context) {
    info!("listening for keypresses...");
    self
      .watch_keystate(ctx.address().into())
      .instrument(Span::current())
      .into_actor(self)
      .spawn(ctx);
  }
}

impl Handler<KeyState> for Listener {
  type Result = ();

  #[instrument(name = "LISTENER", skip(self, msg, _ctx))]
  fn handle(&mut self, msg: KeyState, _ctx: &mut Self::Context) -> Self::Result {
    let key = convert_device_query_keys_to_key(&msg.0);
    if key != Key::default() && Some(key) != self.prev_key {
      self.prev_key = Some(key);
      self.filter_rec.do_send(KeyDetected(key));
    }
  }
}

fn convert_device_query_keys_to_key(pressed: &[Keycode]) -> Key {
  let mut repr: u64 = 0;
  for &kc in pressed {
    if let Some(offset) = key_offset(kc) {
      repr |= 1 << offset;
    }
  }
  Key { repr }
}

fn key_offset(kc: Keycode) -> Option<u8> {
  let i = match kc {
    // modifiers
    Keycode::LControl | Keycode::RControl => 0,
    Keycode::LShift | Keycode::RShift => 1,
    Keycode::LAlt | Keycode::RAlt => 2,
    Keycode::Command | Keycode::LMeta | Keycode::RMeta | Keycode::LOption | Keycode::ROption => 3,
    // letters A..Z => 4..(4+26)-1
    Keycode::A => 4, /*+ 0*/
    Keycode::B => 4 + 1,
    Keycode::C => 4 + 2,
    Keycode::D => 4 + 3,
    Keycode::E => 4 + 4,
    Keycode::F => 4 + 5,
    Keycode::G => 4 + 6,
    Keycode::H => 4 + 7,
    Keycode::I => 4 + 8,
    Keycode::J => 4 + 9,
    Keycode::K => 4 + 10,
    Keycode::L => 4 + 11,
    Keycode::M => 4 + 12,
    Keycode::N => 4 + 13,
    Keycode::O => 4 + 14,
    Keycode::P => 4 + 15,
    Keycode::Q => 4 + 16,
    Keycode::R => 4 + 17,
    Keycode::S => 4 + 18,
    Keycode::T => 4 + 19,
    Keycode::U => 4 + 20,
    Keycode::V => 4 + 21,
    Keycode::W => 4 + 22,
    Keycode::X => 4 + 23,
    Keycode::Y => 4 + 24,
    Keycode::Z => 4 + 25,
    // digits 0..9 => bits (4+26)..(4+26+9)
    Keycode::Key0 => 4 + 26, /* + 0*/
    Keycode::Key1 => 4 + 26 + 1,
    Keycode::Key2 => 4 + 26 + 2,
    Keycode::Key3 => 4 + 26 + 3,
    Keycode::Key4 => 4 + 26 + 4,
    Keycode::Key5 => 4 + 26 + 5,
    Keycode::Key6 => 4 + 26 + 6,
    Keycode::Key7 => 4 + 26 + 7,
    Keycode::Key8 => 4 + 26 + 8,
    Keycode::Key9 => 4 + 26 + 9,
    // whatever else...
    _ => return None,
  };
  Some(i)
}
