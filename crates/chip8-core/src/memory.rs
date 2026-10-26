use serde::{Deserialize, Serialize};
use crate::error::{Chip8Error, Result };

const MEMORY_SIZE: usize = 4096;
const ROM_START: usize = 0x200;

/// CHIP-8 addresses are 12 bits wide (0x000 to 0xFFF), but `addr` is a `u16`,
/// so values above 0xFFF can be passed in, for example, when a program adds
/// to the `I` register. The upper 4 bits are discarded by masking with
/// `0xFFF`, so the address wraps around instead of going out of bounds.
const ADDR_MASK: u16 = 0xFFF;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    #[serde(with = "serde_arrays")]
    bytes: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        use crate::fonts::{ FONT, FONT_START };

        let mut mem = Self {
            bytes: [0; MEMORY_SIZE]
        };

        // Copy the fonts to memory
        mem.bytes[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);

        mem
    }

    /// Reads one byte from memory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chip8_core::memory::Memory;
    /// let mut mem = Memory::new();
    /// mem.write(0x300, 0xAB);
    /// assert_eq!(mem.read(0x300), 0xAB);
    /// assert_eq!(mem.read(0x1300), 0xAB); // 0x1300 wraps to 0x300
    /// ```
    pub fn read(&self, addr: u16) -> u8 {
        self.bytes[(addr & ADDR_MASK) as usize]
    }

    /// Write one byte to memory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chip8_core::memory::Memory;
    /// let mut mem = Memory::new();
    /// mem.write(0x300, 0xAB);
    /// assert_eq!(mem.read(0x300), 0xAB);
    /// assert_eq!(mem.read(0x1300), 0xAB); // 0x1300 wraps to 0x300
    /// ```
    pub fn write(&mut self, addr: u16, value: u8) {
        self.bytes[(addr & ADDR_MASK) as usize] = value;
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        let max = MEMORY_SIZE - ROM_START;
        if rom.len() > max {
            return Err(Chip8Error::RomTooLarge { size: rom.len(), max });
        }

        self.bytes[ROM_START..ROM_START + rom.len()].copy_from_slice(rom);

        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::{ FONT_START, FONT };

    #[test]
    fn font_is_loaded() {
        let mem = Memory::new();
        assert_eq!(mem.read(FONT_START as u16), FONT[0]);
    }

    #[test]
    fn rom_is_copied_to_rom_start() {
        let mut mem = Memory::new();
        mem.load_rom(&[0xA2, 0x2A]).unwrap();
        assert_eq!(mem.read(ROM_START as u16), 0xA2);
        assert_eq!(mem.read(ROM_START as u16 + 1), 0x2A);
    }

    #[test]
    fn oversized_rom_is_rejected() {
        let mut mem = Memory::new();
        let err = mem.load_rom(&[0u8; 4000]).unwrap_err();
        assert_eq!(err, Chip8Error::RomTooLarge { size: 4000, max: 3584 });
    }
}