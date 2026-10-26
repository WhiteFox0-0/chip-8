use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Timer {
    delay: u8,
    sound: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Counts both timers down by one, stopping at zero.
    /// Call this exactly 60 times per second.
    pub fn tick(&mut self) {
        self.delay = self.delay.saturating_sub(1);
        self.sound = self.sound.saturating_sub(1);
    }

    pub fn delay(&self) -> u8 {
        self.delay
    }

    pub fn set_delay(&mut self, value: u8) {
        self.delay = value;
    }

    pub fn set_sound(&mut self, value: u8) {
        self.sound = value;
    }

    /// True while the beep should be playing.
    pub fn sound_active(&self) -> bool {
        self.sound > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_and_stops_at_zero() {
        let mut t = Timer::new();
        t.set_delay(2);
        t.set_sound(1);

        t.tick();
        assert_eq!(t.delay(), 1);
        assert!(!t.sound_active());

        t.tick();
        t.tick();                      // extra tick must not wrap to 255
        assert_eq!(t.delay(), 0);
    }
}