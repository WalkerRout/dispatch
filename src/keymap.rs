use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::key::Key;
use crate::script::Script;

#[derive(Debug, Serialize, Deserialize)]
struct Keybind {
  keys: Vec<String>,
  script: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeymapFormat {
  keybinds: Vec<Keybind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keymap(pub HashMap<Key, Script>);

pub fn parse_json(json_bytes: &[u8]) -> Result<Keymap, anyhow::Error> {
  let parsed: KeymapFormat = serde_json::from_slice(json_bytes)?;
  let mut map: HashMap<Key, Script> = HashMap::new();
  for keybind in parsed.keybinds {
    let key = Key::from_names(keybind.keys);
    let script = keybind.script;
    map.insert(key, script);
  }
  Ok(Keymap(map))
}

impl Deref for Keymap {
  type Target = HashMap<Key, Script>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Keymap {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
