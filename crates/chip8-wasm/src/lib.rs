use chip8_core::cpu::CPU;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Emulator {
    cpu: CPU,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { cpu: CPU::new() }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), JsValue> {
        self.cpu
            .load_rom(rom)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn key(&mut self, key: u8, down: bool) {
        self.cpu.set_key(key, down);
    }

    pub fn step(&mut self) {
        self.cpu.step();
    }

    pub fn tick(&mut self) {
        self.cpu.tick_timers();
    }

    pub fn sound(&self) -> bool {
        self.cpu.sound_active()
    }

    pub fn framebuffer(&self) -> Vec<u8> {
        let mut out = vec![0u8; 256];
        for (pixel, &on) in self.cpu.framebuffer().iter().enumerate() {
            if on {
                out[pixel >> 3] |= 1 << (7 - (pixel & 7));
            }
        }
        out
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
