use serde::{Deserialize, Serialize};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

/// The 64x32 monochrome screen. `true` means the pixel is lit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    #[serde(with = "serde_arrays")]
    pixels: [bool; WIDTH * HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        Self { pixels: [false; WIDTH * HEIGHT] }
    }

    /// Turns every pixel off (opcode 00E0).
    pub fn clear(&mut self) {
        self.pixels.fill(false);
    }

    /// Returns the pixel at (x, y). Out-of-range coordinates read as off.
    pub fn get(&self, x: usize, y: usize) -> bool {
        x < WIDTH && y < HEIGHT && self.pixels[y * WIDTH + x]
    }

    /// The whole screen, row by row, for the frontend to draw.
    pub fn pixels(&self) -> &[bool] {
        &self.pixels
    }

    /// XORs a sprite onto the screen and returns true if any lit pixel was
    /// turned off (a collision).
    ///
    /// Each byte of `sprite` is one 8-pixel row, the most significant bit first.
    /// The starting position wraps around the screen, but pixels that run
    /// past the right or bottom edge are clipped, not wrapped.
    pub fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let start_x = x as usize % WIDTH;
        let start_y = y as usize % HEIGHT;
        let mut collision = false;

        for (row, byte) in sprite.iter().enumerate() {
            let py = start_y + row;
            if py >= HEIGHT {
                break;
            }
            for col in 0..8 {
                let px = start_x + col;
                if px >= WIDTH {
                    break;
                }
                if byte & (0x80 >> col) != 0 {
                    let idx = py * WIDTH + px;
                    if self.pixels[idx] {
                        collision = true;
                    }
                    self.pixels[idx] ^= true;
                }
            }
        }

        collision
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_a_row() {
        let mut d = Display::new();
        let hit = d.draw_sprite(0, 0, &[0b1010_0000]);
        assert!(!hit);
        assert!(d.get(0, 0));
        assert!(!d.get(1, 0));
        assert!(d.get(2, 0));
    }

    #[test]
    fn drawing_twice_erases_and_reports_collision() {
        let mut d = Display::new();
        d.draw_sprite(5, 5, &[0xFF]);
        let hit = d.draw_sprite(5, 5, &[0xFF]);
        assert!(hit);
        assert!(!d.get(5, 5));
    }

    #[test]
    fn start_wraps_but_pixels_clip() {
        let mut d = Display::new();
        d.draw_sprite(66, 0, &[0x80]);        // 66 % 64 = 2
        assert!(d.get(2, 0));

        let mut d = Display::new();
        d.draw_sprite(60, 0, &[0xFF]);        // columns 64+ are cut off
        assert!(d.get(63, 0));
        assert!(!d.get(0, 0));                // did not wrap to the left edge
    }
}