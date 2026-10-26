#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    ClearScreen,                            // 00E0
    Return,                                 // 00EE
    Jump { addr: u16 },                     // 1NNN
    Call { addr: u16 },                     // 2NNN

    /// SKIP next instruction if register Vx = kk
    SkipEqImm { x: usize, kk: u8 },         // 3XKK

    /// SKIP next instruction if register Vx != kk
    SkipNeImm { x: usize, kk: u8 },         // 4XKK

    /// SKIP next instruction if register Vx = Vy
    SkipEqReg { x: usize, y: usize },       // 5XY0

    /// SET value kk in Register Vx
    SetImm { x: usize, kk: u8 },           // 6XKK

    /// ADD value kk to Register Vx
    AddImm { x: usize, kk: u8 },           // 7XKK

    /// SET value in Register Vx = Register Vy
    SetReg { x: usize, y: usize},          // 8XY0

    /// SET Register Vx = Vx OR Vy
    OrReg { x: usize, y: usize },          // 8XY1

    /// SET Register Vx = Vx AND Vy
    AndReg { x: usize, y: usize },         // 8XY2

    /// SET Register Vx = Vx XOR Vy
    XorReg { x: usize, y: usize },         // 8XY3

    /// ADD values of Register Vx and Vy
    /// stores the carry in Vf
    AddReg { x: usize, y: usize },         // 8XY4

    /// SUB values of Register Vx and Vy
    /// Vf stores the sign
    SubReg { x: usize, y: usize },         // 8XY5

    /// SHIFT Register Vx right by 1
    /// Vf stores the bit that was shifted out
    ShiftRight { x: usize, y: usize },     // 8XY6

    /// SET Register Vx = Vy - Vx
    /// Vf = 1 if there was no borrow (Vy >= Vx), otherwise 0
    SubRevReg { x: usize, y: usize },      // 8XY7

    /// SHIFT Register Vx left by 1
    /// Vf stores the bit that was shifted out
    ShiftLeft { x: usize, y: usize },      // 8XYE

    /// SKIP next instruction if register Vx != Vy
    SkipNeReg { x: usize, y: usize },      // 9XY0

    /// SET Register I = addr
    SetIndex { addr: u16 },                // ANNN

    /// JUMP to addr + V0
    JumpOffset { addr: u16 },              // BNNN

    /// SET Register Vx = random byte AND kk
    Random { x: usize, kk: u8 },           // CXKK

    /// DRAW an n-byte sprite from memory at I, at position (Vx, Vy)
    /// Vf = 1 if any lit pixel was turned off (collision), otherwise 0
    Draw { x: usize, y: usize, n: u8 },    // DXYN

    /// SKIP next instruction if the key with the value in Vx is pressed
    SkipKeyDown { x: usize },              // EX9E

    /// SKIP next instruction if the key with the value in Vx is NOT pressed
    SkipKeyUp { x: usize },                // EXA1

    /// SET Register Vx = delay timer
    GetDelay { x: usize },                 // FX07

    /// WAIT for a key press, then store the key in Vx
    WaitKey { x: usize },                  // FX0A

    /// SET delay timer = Vx
    SetDelay { x: usize },                 // FX15

    /// SET sound timer = Vx
    SetSound { x: usize },                 // FX18

    /// ADD Vx to Register I
    AddIndex { x: usize },                 // FX1E

    /// SET Register I = address of the font glyph for the digit in Vx
    FontChar { x: usize },                 // FX29

    /// STORE the decimal digits of Vx in memory at I, I+1, I+2
    StoreBcd { x: usize },                 // FX33

    /// STORE registers V0 through Vx in memory starting at I
    StoreRegs { x: usize },                // FX55

    /// LOAD registers V0 through Vx from memory starting at I
    LoadRegs { x: usize },                 // FX65

    /// Any opcode that matches none of the above
    Unknown(u16),
}

impl Opcode {
    /// Turns a raw 16-bit instruction into an `Opcode`. Never fails:
    /// anything unrecognised becomes `Opcode::Unknown`.
    pub fn decode(op: u16) -> Self {
        let x = ((op >> 8) & 0xF) as usize;
        let y = ((op >> 4) & 0xF) as usize;
        let n = (op & 0xF) as u8;
        let kk = (op & 0xFF) as u8;
        let addr = op & 0xFFF;

        match ((op >> 12) as u8, x, y, n) {
            (0x0, 0, 0xE, 0x0) => Self::ClearScreen,
            (0x0, 0, 0xE, 0xE) => Self::Return,
            (0x1, ..) => Self::Jump { addr },
            (0x2, ..) => Self::Call { addr },
            (0x3, ..) => Self::SkipEqImm { x, kk },
            (0x4, ..) => Self::SkipNeImm { x, kk },
            (0x5, _, _, 0x0) => Self::SkipEqReg { x, y },
            (0x6, ..) => Self::SetImm { x, kk },
            (0x7, ..) => Self::AddImm { x, kk },
            (0x8, _, _, 0x0) => Self::SetReg { x, y },
            (0x8, _, _, 0x1) => Self::OrReg { x, y },
            (0x8, _, _, 0x2) => Self::AndReg { x, y },
            (0x8, _, _, 0x3) => Self::XorReg { x, y },
            (0x8, _, _, 0x4) => Self::AddReg { x, y },
            (0x8, _, _, 0x5) => Self::SubReg { x, y },
            (0x8, _, _, 0x6) => Self::ShiftRight { x, y },
            (0x8, _, _, 0x7) => Self::SubRevReg { x, y },
            (0x8, _, _, 0xE) => Self::ShiftLeft { x, y },
            (0x9, _, _, 0x0) => Self::SkipNeReg { x, y },
            (0xA, ..) => Self::SetIndex { addr },
            (0xB, ..) => Self::JumpOffset { addr },
            (0xC, ..) => Self::Random { x, kk },
            (0xD, ..) => Self::Draw { x, y, n },
            (0xE, _, 0x9, 0xE) => Self::SkipKeyDown { x },
            (0xE, _, 0xA, 0x1) => Self::SkipKeyUp { x },
            (0xF, _, 0x0, 0x7) => Self::GetDelay { x },
            (0xF, _, 0x0, 0xA) => Self::WaitKey { x },
            (0xF, _, 0x1, 0x5) => Self::SetDelay { x },
            (0xF, _, 0x1, 0x8) => Self::SetSound { x },
            (0xF, _, 0x1, 0xE) => Self::AddIndex { x },
            (0xF, _, 0x2, 0x9) => Self::FontChar { x },
            (0xF, _, 0x3, 0x3) => Self::StoreBcd { x },
            (0xF, _, 0x5, 0x5) => Self::StoreRegs { x },
            (0xF, _, 0x6, 0x5) => Self::LoadRegs { x },
            _ => Self::Unknown(op),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_basic_instructions() {
        assert_eq!(Opcode::decode(0x00E0), Opcode::ClearScreen);
        assert_eq!(Opcode::decode(0x1228), Opcode::Jump { addr: 0x228 });
        assert_eq!(Opcode::decode(0x6A05), Opcode::SetImm { x: 0xA, kk: 0x05 });
        assert_eq!(Opcode::decode(0x8014), Opcode::AddReg { x: 0, y: 1 });
        assert_eq!(Opcode::decode(0xD123), Opcode::Draw { x: 1, y: 2, n: 3 });
    }

    #[test]
    fn unrecognised_becomes_unknown() {
        assert_eq!(Opcode::decode(0x0123), Opcode::Unknown(0x0123));
        assert_eq!(Opcode::decode(0x5121), Opcode::Unknown(0x5121));
    }
}