use serde::{Serialize, Deserialize};
use crate::memory::Memory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registers {
    /// 16 8-bit General-purpose registers
    /// (V0 - VF). VF used as a flag by some instructions
    pub v: [u8; 16],

    /// Index Register, used to point to location in memory
    /// Only lower 12-bits are used out of 16-bits (For 4096 addresses)
    pub i: u16,

    /// 16-bit Program counter
    pub pc: u16,

    /// 8-bit Stack Pointer (SP)
    pub sp: u8,

    /// Delay Timer. Decrements at a rate of 60Hz when greater than 0
    pub delay_timer: u8,

    /// Sound Timer. Decrements at a rate of 60Hz when greater than 0
    pub sound_timer: u8,
}


impl Registers {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}