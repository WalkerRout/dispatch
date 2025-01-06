use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
  /// Keys can modify each other; need to store a bitfield of possible selected values
  /// - 4 bits modifiers + 10 bits digits + 26 bits letters = 40 bits needed -> store packed in first bits of u64
  /// - 0b00000000 00000000 00000000 0ddddddd ddaaaaaa aaaaaaaa aaaaaaaa aaaammmm
  pub repr: u64,
}

impl Key {
  pub fn from_names(key_names: impl IntoIterator<Item = String>) -> Self {
    let mut repr: u64 = 0;
    for mut key_name in key_names {
      key_name.make_ascii_lowercase();
      match key_name.trim() {
        "ctrl" | "control" => repr |= 1 << 0,
        "shift" => repr |= 1 << 1,
        "alt" | "alternate" | "option" => repr |= 1 << 2,
        "super" | "win" | "windows" | "cmd" | "command" => repr |= 1 << 3,
        key if key.len() == 1 => {
          let char_code = key.chars().next().unwrap() as u64;
          match char_code as u8 {
            b'a'..=b'z' => repr |= 1 << (4 + (char_code - b'a' as u64)),
            b'0'..=b'9' => repr |= 1 << (4 + 26 + (char_code - b'0' as u64)),
            _ => {}
          }
        }
        key => panic!("did not recognize key: {key}"),
      }
    }
    Key { repr }
  }
}
