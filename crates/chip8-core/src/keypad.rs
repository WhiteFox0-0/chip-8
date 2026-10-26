use serde::{Deserialize, Serialize};

/// State of the 16-key hex keypad (keys 0x0 to 0xF).
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Keypad {
    keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks a key as pressed or released. Only the low 4 bits of `key` are used.
    pub fn set(&mut self, key: u8, pressed: bool) {
        self.keys[(key & 0xF) as usize] = pressed;
    }

    /// Returns true if the key is currently held down.
    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys[(key & 0xF) as usize]
    }

    /// Returns the lowest-numbered key that is held down, if any.
    pub fn first_pressed(&self) -> Option<u8> {
        self.keys.iter().position(|&k| k).map(|i| i as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_and_release() {
        let mut kp = Keypad::new();
        assert!(!kp.is_pressed(0xA));
        kp.set(0xA, true);
        assert!(kp.is_pressed(0xA));
        kp.set(0xA, false);
        assert_eq!(kp.first_pressed(), None);
    }

    #[test]
    fn key_index_is_masked() {
        let mut kp = Keypad::new();
        kp.set(0x1F, true);               // wraps to 0xF
        assert!(kp.is_pressed(0xF));
    }
}