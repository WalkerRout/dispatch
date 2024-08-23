use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

use tokio::task;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
  /// Keys can modify each other; need to store a bitfield of possible selected values
  /// - 4 bits modifiers + 10 bits digits + 26 bits letters = 40 bits needed -> store in first bits of u64
  /// - modifiers bits are in the following order; control, shift, alt, super
  pub repr: u64,
}

impl Key {
  pub async fn from_async_key_state() -> Self {
    let shift_pressed =
      task::spawn_blocking(|| unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0 })
        .await
        .unwrap();

    let ctrl_pressed = task::spawn_blocking(|| unsafe {
      GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000 != 0
    })
    .await
    .unwrap();

    #[rustfmt::skip]
    let alt_pressed = task::spawn_blocking(|| unsafe {
      GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000 != 0
    })
    .await
    .unwrap();

    let super_pressed = task::spawn_blocking(|| unsafe {
      GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000 != 0
        || GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000 != 0
    })
    .await
    .unwrap();

    let mut key = Key { repr: 0 };

    key.repr |= ctrl_pressed as u64;
    key.repr |= (shift_pressed as u64) << 1;
    key.repr |= (alt_pressed as u64) << 2;
    key.repr |= (super_pressed as u64) << 3;

    for (i, vk) in (b'A'..=b'Z').chain(b'1'..=b'9').enumerate() {
      let key_pressed =
        task::spawn_blocking(move || unsafe { GetAsyncKeyState(vk as i32) as u16 & 0x8000 != 0 })
          .await
          .unwrap();
      key.repr |= (key_pressed as u64) << (4 + i);
    }

    key
  }

  pub fn from_names<S: AsRef<str>>(key_names: &[S]) -> Self {
    let mut repr: u64 = 0;
    for key_name in key_names {
      let key_name = key_name.as_ref();
      match key_name {
        "Ctrl" => repr |= 1 << 0,
        "Shift" => repr |= 1 << 1,
        "Alt" => repr |= 1 << 2,
        "Super" => repr |= 1 << 3,
        key if key.len() == 1 => {
          let char_code = key.chars().next().unwrap() as u64;
          match char_code as u8 {
            b'A'..=b'Z' => repr |= 1 << (4 + (char_code - b'A' as u64)),
            b'0'..=b'9' => repr |= 1 << (14 + (char_code - b'0' as u64)),
            _ => {}
          }
        }
        _ => {}
      }
    }
    Key { repr }
  }
}
