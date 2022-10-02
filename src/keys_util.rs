use std::time::{Duration, SystemTime};

use rltk::VirtualKeyCode;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum KeyState {
    PRESSED,
    HOLD,
    #[default]
    RELEASED,
}

/// Key press with a repeat delay
#[derive(Debug, Default)]
pub struct KeyPress {
    pub min_delay_ms: u64,
    pub last_press_time: Option<SystemTime>,
    /// True if key was just pressed and needs to wait for repeat delay, else false
    pub state: KeyState,
    pub repeat_delay_ms: u64,
}

impl KeyPress {
    pub fn new(min_delay_ms: u64, repeat_delay_ms: u64) -> Self {
        KeyPress {
            min_delay_ms,
            last_press_time: None,
            state: KeyState::RELEASED,
            repeat_delay_ms,
        }
    }
}

/// Tries to press a key, checking its KeyPress configuration and updating it
///
/// Note: Rltk INPUT pressed key set only contains those pressed all at once until released.
/// Meaning if you press multiple keys and hold, then press another key afterwards, it won't be
/// included in the key set and won't register until the initial key set is released.
pub fn try_press(key: VirtualKeyCode, key_press: Option<&mut KeyPress>) -> bool {
    let input = rltk::INPUT.lock();
    println!("{:#?}", input.key_pressed_set());
    if input.is_key_pressed(key) {
        return if let Some(key_press) = key_press {
            can_press(key_press)
        } else {
            true
        };
    }

    if let Some(key_press) = key_press {
        key_press.state = KeyState::RELEASED;
    }
    false
}

// Ideally this should be based on tick time, not system time!
fn can_press(key: &mut KeyPress) -> bool {
    let current_time = SystemTime::now();
    if let Some(last_press_time) = key.last_press_time {
        let elapsed = current_time.duration_since(last_press_time).unwrap();

        if key.state == KeyState::PRESSED {
            // Repeat delay
            if elapsed < Duration::from_millis(key.repeat_delay_ms) {
                return false;
            } else {
                key.state = KeyState::HOLD;
            }
        }

        // Constrains speed of key presses
        if elapsed < Duration::from_millis(key.min_delay_ms) {
            return false;
        }
    }

    key.last_press_time = Some(current_time);
    if key.state == KeyState::RELEASED {
        key.state = KeyState::PRESSED;
    }
    true
}
