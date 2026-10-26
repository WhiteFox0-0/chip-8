use serde::{Deserialize, Serialize};
use crate::display::Display;
use crate::error::{ Chip8Error, Result };
use crate::fonts::FONT_START;
use crate::keypad::Keypad;
use crate::memory::Memory;
use crate::opcode::Opcode;
use crate::register::Registers;
use crate::timers::Timer;


const STACK_SIZE: usize = 16;

// Quirks (original COSMAC VIP behavior). Flip these if a test ROM disagrees.
const SHIFT_USES_VY: bool = true;
const LOAD_STORE_INCREMENTS_I: bool = true;
const LOGIC_RESETS_VF: bool = true;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CPU {
    memory: Memory,
    reg: Registers,
    display: Display,
    keypad: Keypad,
    timer: Timer,
    stack: [u16; 16],
}

impl CPU {
    pub fn new() -> Self{
        Self::default()
    }

    /// Copies a ROM into memory at the entry point (0x200).
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        self.memory.load_rom(rom)
    }

    /// Presses or releases a key on the 16-key keypad.
    pub fn set_key(&mut self, key: u8, pressed: bool) {
        self.keypad.set(key, pressed);
    }

    /// Counts the delay and sound timers down by one. Call at 60 Hz.
    pub fn tick_timers(&mut self) {
        self.timer.tick();
    }

    /// The 64x32 screen, row by row. `true` means the pixel is lit.
    pub fn framebuffer(&self) -> &[bool] {
        self.display.pixels()
    }

    /// True while the sound timer is counting down.
    pub fn sound_active(&self) -> bool {
        self.timer.sound_active()
    }

    /// Skips the next instruction.
    fn skip(&mut self) {
        self.reg.pc = self.reg.pc.wrapping_add(2);
    }

    pub fn step(&mut self) {
        let instr = self.fetch();
        let opcode = Self::decode(instr);
        let _ = self.execute(opcode);
    }

    fn fetch(&mut self) -> u16 {
        let hi = self.memory.read(self.reg.pc) as u16;
        let lo = self.memory.read(self.reg.pc.wrapping_add(1)) as u16;
        self.reg.pc = self.reg.pc.wrapping_add(2);
        (hi << 8) | lo
    }

    fn decode(instr: u16) -> Opcode{
        Opcode::decode(instr)
    }

    fn execute(&mut self, opcode: Opcode) -> Result<()> {
        match opcode {
            // --- flow control ---
            Opcode::ClearScreen => self.display.clear(),
            Opcode::Return => {
                if self.reg.sp == 0 {
                    return Err(Chip8Error::StackUnderflow);
                }
                self.reg.sp -= 1;
                self.reg.pc = self.stack[self.reg.sp as usize];
            }
            Opcode::Jump { addr } => self.reg.pc = addr,
            Opcode::Call { addr } => {
                if self.reg.sp as usize >= STACK_SIZE {
                    return Err(Chip8Error::StackOverflow);
                }
                self.stack[self.reg.sp as usize] = self.reg.pc;
                self.reg.sp += 1;
                self.reg.pc = addr;
            }
            Opcode::JumpOffset { addr } => {
                self.reg.pc = addr.wrapping_add(self.reg.v[0] as u16);
            }

            // --- conditional skips ---
            Opcode::SkipEqImm { x, kk } => if self.reg.v[x] == kk { self.skip() },
            Opcode::SkipNeImm { x, kk } => if self.reg.v[x] != kk { self.skip() },
            Opcode::SkipEqReg { x, y } => if self.reg.v[x] == self.reg.v[y] { self.skip() },
            Opcode::SkipNeReg { x, y } => if self.reg.v[x] != self.reg.v[y] { self.skip() },

            // --- loads and adds ---
            Opcode::SetImm { x, kk } => self.reg.v[x] = kk,
            Opcode::AddImm { x, kk } => {
                self.reg.v[x] = self.reg.v[x].wrapping_add(kk);   // no carry flag
            }
            Opcode::SetReg { x, y } => self.reg.v[x] = self.reg.v[y],

            // --- logic ---
            Opcode::OrReg { x, y } => {
                self.reg.v[x] |= self.reg.v[y];
                if LOGIC_RESETS_VF { self.reg.v[0xF] = 0; }
            }
            Opcode::AndReg { x, y } => {
                self.reg.v[x] &= self.reg.v[y];
                if LOGIC_RESETS_VF { self.reg.v[0xF] = 0; }
            }
            Opcode::XorReg { x, y } => {
                self.reg.v[x] ^= self.reg.v[y];
                if LOGIC_RESETS_VF { self.reg.v[0xF] = 0; }
            }

            // --- arithmetic (flag is set AFTER the result is written) ---
            Opcode::AddReg { x, y } => {
                let (result, carry) = self.reg.v[x].overflowing_add(self.reg.v[y]);
                self.reg.v[x] = result;
                self.reg.v[0xF] = carry as u8;
            }
            Opcode::SubReg { x, y } => {
                let (result, borrow) = self.reg.v[x].overflowing_sub(self.reg.v[y]);
                self.reg.v[x] = result;
                self.reg.v[0xF] = !borrow as u8;
            }
            Opcode::SubRevReg { x, y } => {
                let (result, borrow) = self.reg.v[y].overflowing_sub(self.reg.v[x]);
                self.reg.v[x] = result;
                self.reg.v[0xF] = !borrow as u8;
            }
            Opcode::ShiftRight { x, y } => {
                let src = if SHIFT_USES_VY { self.reg.v[y] } else { self.reg.v[x] };
                self.reg.v[x] = src >> 1;
                self.reg.v[0xF] = src & 1;
            }
            Opcode::ShiftLeft { x, y } => {
                let src = if SHIFT_USES_VY { self.reg.v[y] } else { self.reg.v[x] };
                self.reg.v[x] = src << 1;
                self.reg.v[0xF] = src >> 7;
            }

            // --- index register ---
            Opcode::SetIndex { addr } => self.reg.i = addr,
            Opcode::AddIndex { x } => {
                self.reg.i = self.reg.i.wrapping_add(self.reg.v[x] as u16);
            }
            Opcode::FontChar { x } => {
                self.reg.i = FONT_START as u16 + (self.reg.v[x] & 0xF) as u16 * 5;
            }

            // --- random ---
            Opcode::Random { x, kk } => self.reg.v[x] = rand::random::<u8>() & kk,

            // --- drawing ---
            Opcode::Draw { x, y, n } => {
                let rows = n as usize;
                let mut sprite = [0u8; 15];
                for (row, byte) in sprite.iter_mut().enumerate().take(rows) {
                    *byte = self.memory.read(self.reg.i.wrapping_add(row as u16));
                }
                let hit = self.display.draw_sprite(
                    self.reg.v[x],
                    self.reg.v[y],
                    &sprite[..rows],
                );
                self.reg.v[0xF] = hit as u8;
            }

            // --- keyboard ---
            Opcode::SkipKeyDown { x } => {
                if self.keypad.is_pressed(self.reg.v[x]) { self.skip() }
            }
            Opcode::SkipKeyUp { x } => {
                if !self.keypad.is_pressed(self.reg.v[x]) { self.skip() }
            }
            Opcode::WaitKey { x } => match self.keypad.first_pressed() {
                Some(key) => self.reg.v[x] = key,
                // No key yet: step back so this same instruction runs again.
                None => self.reg.pc = self.reg.pc.wrapping_sub(2),
            },

            // --- timers ---
            Opcode::GetDelay { x } => self.reg.v[x] = self.timer.delay(),
            Opcode::SetDelay { x } => self.timer.set_delay(self.reg.v[x]),
            Opcode::SetSound { x } => self.timer.set_sound(self.reg.v[x]),

            // --- memory ---
            Opcode::StoreBcd { x } => {
                let v = self.reg.v[x];
                self.memory.write(self.reg.i, v / 100);
                self.memory.write(self.reg.i.wrapping_add(1), (v / 10) % 10);
                self.memory.write(self.reg.i.wrapping_add(2), v % 10);
            }
            Opcode::StoreRegs { x } => {
                for r in 0..=x {
                    self.memory.write(self.reg.i.wrapping_add(r as u16), self.reg.v[r]);
                }
                if LOAD_STORE_INCREMENTS_I {
                    self.reg.i = self.reg.i.wrapping_add(x as u16 + 1);
                }
            }
            Opcode::LoadRegs { x } => {
                for r in 0..=x {
                    self.reg.v[r] = self.memory.read(self.reg.i.wrapping_add(r as u16));
                }
                if LOAD_STORE_INCREMENTS_I {
                    self.reg.i = self.reg.i.wrapping_add(x as u16 + 1);
                }
            }

            Opcode::Unknown(op) => return Err(Chip8Error::UnknownOpcode(op)),
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{HEIGHT, WIDTH};

    fn with(program: &[u8]) -> CPU {
        let mut cpu = CPU::new();
        cpu.load_rom(program).unwrap();
        cpu
    }

    fn run(cpu: &mut CPU, steps: usize) {
        for _ in 0..steps {
            cpu.step();
        }
    }

    #[test]
    fn rom_is_loaded_at_entry_point() {
        let cpu = with(&[0xAB, 0xCD]);
        assert_eq!(cpu.memory.read(0x200), 0xAB);
        assert_eq!(cpu.memory.read(0x201), 0xCD);
    }

    #[test]
    fn oversized_rom_is_rejected_by_cpu() {
        let mut cpu = CPU::new();
        assert!(cpu.load_rom(&[0u8; 4000]).is_err());
    }

    #[test]
    fn clear_screen_blanks_the_display() {
        let mut cpu = with(&[0x00, 0xE0]);
        cpu.display.draw_sprite(0, 0, &[0xFF]);
        assert!(cpu.framebuffer().iter().any(|&p| p));

        cpu.step();

        assert!(cpu.framebuffer().iter().all(|&p| !p));
    }

    #[test]
    fn call_pushes_and_return_pops() {
        let mut cpu = with(&[
            0x22, 0x06, // 0x200: call 0x206
            0x12, 0x04, // 0x202: jump 0x204
            0x00, 0x00, // 0x204
            0x60, 0x42, // 0x206: V0 = 0x42
            0x00, 0xEE, // 0x208: return
        ]);

        cpu.step();
        assert_eq!(cpu.reg.sp, 1);
        assert_eq!(cpu.reg.pc, 0x206);

        cpu.step();
        cpu.step();

        assert_eq!(cpu.reg.sp, 0);
        assert_eq!(cpu.reg.pc, 0x202);
        assert_eq!(cpu.reg.v[0], 0x42);
    }

    #[test]
    fn jump_and_jump_offset_set_pc() {
        let mut cpu = with(&[0x12, 0x34]);
        cpu.step();
        assert_eq!(cpu.reg.pc, 0x234);

        let mut cpu = with(&[0x60, 0x10, 0xB2, 0x00]);
        cpu.step();
        cpu.step();
        assert_eq!(cpu.reg.pc, 0x210);
    }

    #[test]
    fn conditional_skips_step_over_next_instruction() {
        let mut cpu = with(&[
            0x60, 0x01, // V0 = 1
            0x30, 0x01, // skip if V0 == 1
            0x60, 0xFF, // skipped
            0x61, 0x02, // V1 = 2
        ]);
        run(&mut cpu, 3);
        assert_eq!((cpu.reg.v[0], cpu.reg.v[1]), (1, 2));

        let mut cpu = with(&[
            0x60, 0x01, // V0 = 1
            0x40, 0x02, // skip if V0 != 2
            0x60, 0xFF, // skipped
            0x61, 0x03, // V1 = 3
        ]);
        run(&mut cpu, 3);
        assert_eq!((cpu.reg.v[0], cpu.reg.v[1]), (1, 3));

        let mut cpu = with(&[
            0x60, 0x05, // V0 = 5
            0x61, 0x05, // V1 = 5
            0x50, 0x10, // skip if V0 == V1
            0x60, 0xFF, // skipped
            0x62, 0x09, // V2 = 9
        ]);
        run(&mut cpu, 4);
        assert_eq!((cpu.reg.v[0], cpu.reg.v[2]), (5, 9));

        let mut cpu = with(&[
            0x60, 0x05, // V0 = 5
            0x61, 0x06, // V1 = 6
            0x90, 0x10, // skip if V0 != V1
            0x60, 0xFF, // skipped
            0x62, 0x07, // V2 = 7
        ]);
        run(&mut cpu, 4);
        assert_eq!((cpu.reg.v[0], cpu.reg.v[2]), (5, 7));
    }

    #[test]
    fn set_and_add_immediate() {
        let mut cpu = with(&[0x60, 0xFF, 0x70, 0x02]);
        cpu.step();
        cpu.step();
        assert_eq!(cpu.reg.v[0], 0x01); // wraps, no carry flag
    }

    #[test]
    fn set_and_logic_register_ops() {
        let mut cpu = CPU::new();
        cpu.reg.v[1] = 0x0F;
        cpu.reg.v[2] = 0xF0;

        cpu.execute(Opcode::SetReg { x: 0, y: 1 }).unwrap();
        assert_eq!(cpu.reg.v[0], 0x0F);

        cpu.reg.v[0xF] = 1;
        cpu.execute(Opcode::OrReg { x: 1, y: 2 }).unwrap();
        assert_eq!(cpu.reg.v[1], 0xFF);
        assert_eq!(cpu.reg.v[0xF], 0);

        cpu.reg.v[1] = 0x0F;
        cpu.reg.v[0xF] = 1;
        cpu.execute(Opcode::AndReg { x: 1, y: 2 }).unwrap();
        assert_eq!(cpu.reg.v[1], 0x00);
        assert_eq!(cpu.reg.v[0xF], 0);

        cpu.reg.v[1] = 0b1100;
        cpu.reg.v[0xF] = 1;
        cpu.execute(Opcode::XorReg { x: 1, y: 2 }).unwrap();
        assert_eq!(cpu.reg.v[1], 0b1100 ^ 0xF0);
        assert_eq!(cpu.reg.v[0xF], 0);
    }

    #[test]
    fn add_and_subtract_flags() {
        let mut cpu = CPU::new();
        cpu.reg.v[1] = 0xFF;
        cpu.reg.v[2] = 1;
        cpu.execute(Opcode::AddReg { x: 1, y: 2 }).unwrap();
        assert_eq!((cpu.reg.v[1], cpu.reg.v[0xF]), (0x00, 1));

        cpu.reg.v[1] = 5;
        cpu.reg.v[2] = 3;
        cpu.execute(Opcode::SubReg { x: 1, y: 2 }).unwrap();
        assert_eq!((cpu.reg.v[1], cpu.reg.v[0xF]), (2, 1));

        cpu.reg.v[1] = 3;
        cpu.reg.v[2] = 5;
        cpu.execute(Opcode::SubReg { x: 1, y: 2 }).unwrap();
        assert_eq!((cpu.reg.v[1], cpu.reg.v[0xF]), (0xFE, 0));

        cpu.reg.v[1] = 3;
        cpu.reg.v[2] = 5;
        cpu.execute(Opcode::SubRevReg { x: 1, y: 2 }).unwrap();
        assert_eq!((cpu.reg.v[1], cpu.reg.v[0xF]), (2, 1));
    }

    #[test]
    fn shifts_use_vy_and_store_shifted_out_bit() {
        let mut cpu = CPU::new();
        cpu.reg.v[1] = 0xFF; // Vx, must be ignored while SHIFT_USES_VY
        cpu.reg.v[2] = 0b101;
        cpu.execute(Opcode::ShiftRight { x: 1, y: 2 }).unwrap();
        assert_eq!((cpu.reg.v[1], cpu.reg.v[0xF]), (0b010, 1));

        cpu.reg.v[1] = 0x00;
        cpu.reg.v[2] = 0x81;
        cpu.execute(Opcode::ShiftLeft { x: 1, y: 2 }).unwrap();
        assert_eq!((cpu.reg.v[1], cpu.reg.v[0xF]), (0x02, 1));
    }

    #[test]
    fn index_ops() {
        let mut cpu = CPU::new();
        cpu.execute(Opcode::SetIndex { addr: 0x300 }).unwrap();
        assert_eq!(cpu.reg.i, 0x300);

        cpu.reg.v[0] = 0x10;
        cpu.execute(Opcode::AddIndex { x: 0 }).unwrap();
        assert_eq!(cpu.reg.i, 0x310);

        cpu.reg.v[0] = 0x0A;
        cpu.execute(Opcode::FontChar { x: 0 }).unwrap();
        assert_eq!(cpu.reg.i, FONT_START as u16 + 0x0A * 5);
    }

    #[test]
    fn random_is_masked() {
        let mut cpu = CPU::new();
        cpu.reg.v[0] = 0xFF;
        cpu.execute(Opcode::Random { x: 0, kk: 0x00 }).unwrap();
        assert_eq!(cpu.reg.v[0], 0x00);

        for _ in 0..32 {
            cpu.execute(Opcode::Random { x: 0, kk: 0x0F }).unwrap();
            assert!(cpu.reg.v[0] <= 0x0F);
        }
    }

    #[test]
    fn draw_sets_and_clears_collision_flag() {
        let mut cpu = with(&[
            0xA2, 0x0A, // I = 0x20A
            0x60, 0x00, // V0 = 0
            0x61, 0x00, // V1 = 0
            0xD0, 0x11, // draw 1 byte
            0xD0, 0x11, // draw again -> collision
            0xFF,       // sprite
        ]);
        run(&mut cpu, 4);

        assert!(cpu.display.get(0, 0));
        assert!(cpu.display.get(7, 0));
        assert_eq!(cpu.reg.v[0xF], 0);

        cpu.step();
        assert!(!cpu.display.get(0, 0));
        assert_eq!(cpu.reg.v[0xF], 1);
    }

    #[test]
    fn key_skips() {
        let mut cpu = with(&[
            0x60, 0x05, // V0 = 5
            0xE0, 0x9E, // skip if key 5 is down
            0x60, 0xFF, // skipped
            0x61, 0x01, // V1 = 1
        ]);
        cpu.set_key(5, true);
        run(&mut cpu, 3);
        assert_eq!((cpu.reg.v[0], cpu.reg.v[1]), (5, 1));

        let mut cpu = with(&[
            0x60, 0x05, // V0 = 5
            0xE0, 0xA1, // skip if key 5 is up
            0x60, 0xFF, // skipped
            0x61, 0x02, // V1 = 2
        ]);
        run(&mut cpu, 3);
        assert_eq!((cpu.reg.v[0], cpu.reg.v[1]), (5, 2));
    }

    #[test]
    fn wait_key_blocks_until_a_key_is_pressed() {
        let mut cpu = with(&[0xF0, 0x0A, 0x61, 0x01]);

        cpu.step();
        assert_eq!(cpu.reg.pc, 0x200); // re-executes until a key arrives

        cpu.set_key(0xA, true);
        cpu.step();
        assert_eq!(cpu.reg.v[0], 0xA);
        assert_eq!(cpu.reg.pc, 0x202);
    }

    #[test]
    fn delay_and_sound_timers() {
        let mut cpu = with(&[
            0x60, 0x09, // V0 = 9
            0xF0, 0x15, // delay = V0
            0x61, 0x03, // V1 = 3
            0xF1, 0x18, // sound = V1
        ]);
        run(&mut cpu, 4);
        assert_eq!(cpu.timer.delay(), 9);
        assert!(cpu.sound_active());

        cpu.tick_timers();
        assert_eq!(cpu.timer.delay(), 8);
        assert!(cpu.sound_active());

        for _ in 0..3 {
            cpu.tick_timers();
        }
        assert!(!cpu.sound_active());
        assert_eq!(cpu.timer.delay(), 5);
    }

    #[test]
    fn get_delay_copies_into_register() {
        let mut cpu = with(&[0x60, 0x06, 0xF0, 0x15, 0xF0, 0x07]);
        run(&mut cpu, 3);
        assert_eq!(cpu.reg.v[0], 6);
    }

    #[test]
    fn store_bcd_writes_digits() {
        let mut cpu = with(&[0x60, 0xEA, 0xA3, 0x00, 0xF0, 0x33]);
        run(&mut cpu, 3);
        assert_eq!(cpu.memory.read(0x300), 2);
        assert_eq!(cpu.memory.read(0x301), 3);
        assert_eq!(cpu.memory.read(0x302), 4);
    }

    #[test]
    fn store_and_load_registers_increment_i() {
        let mut cpu = with(&[
            0x60, 0x01, // V0 = 1
            0x61, 0x02, // V1 = 2
            0x62, 0x03, // V2 = 3
            0xA3, 0x00, // I = 0x300
            0xF2, 0x55, // store V0..V2
        ]);
        run(&mut cpu, 5);
        assert_eq!(cpu.memory.read(0x300), 1);
        assert_eq!(cpu.memory.read(0x301), 2);
        assert_eq!(cpu.memory.read(0x302), 3);
        assert_eq!(cpu.reg.i, 0x303);

        let mut cpu = with(&[0xA3, 0x00, 0xF2, 0x65]);
        cpu.memory.write(0x300, 7);
        cpu.memory.write(0x301, 8);
        cpu.memory.write(0x302, 9);
        run(&mut cpu, 2);
        assert_eq!((cpu.reg.v[0], cpu.reg.v[1], cpu.reg.v[2]), (7, 8, 9));
        assert_eq!(cpu.reg.i, 0x303);
    }

    #[test]
    fn framebuffer_matches_display_size() {
        let cpu = CPU::new();
        assert_eq!(cpu.framebuffer().len(), WIDTH * HEIGHT);
    }

    #[test]
    fn execute_reports_flow_errors() {
        let mut cpu = CPU::new();
        assert_eq!(cpu.execute(Opcode::Return), Err(Chip8Error::StackUnderflow));

        cpu.reg.sp = STACK_SIZE as u8;
        assert_eq!(
            cpu.execute(Opcode::Call { addr: 0x300 }),
            Err(Chip8Error::StackOverflow)
        );

        assert_eq!(
            cpu.execute(Opcode::decode(0x0123)),
            Err(Chip8Error::UnknownOpcode(0x0123))
        );
    }
}
