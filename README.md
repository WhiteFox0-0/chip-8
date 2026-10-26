# chip8

A CHIP-8 emulator in Rust. CHIP-8 is a tiny virtual machine from the 1970s: 4 KB of memory, 16 registers, a 64x32 monochrome display, and about 35 instructions. It is small enough to build in a weekend, and it covers the core ideas behind every emulator: fetch, decode, execute, timers, input, and a framebuffer.

Play in the browser: <https://whitefox0-0.github.io/chip-8/>

The emulator core is a separate library with no window or audio code, so it can be reused in other frontends (web, desktop, terminal). The native server and the static WebAssembly build share the same core.

## Run

Native HTTP server:

```bash
cargo run --release -p chip8-web
```

Open <http://127.0.0.1:8080> (set `PORT` to change it) and pick a card. Previews are generated live by running each ROM.

Static browser build:

```bash
wasm-pack build crates/chip8-wasm --target web --out-dir web/pkg
mkdir -p crates/chip8-wasm/web/roms
cp -r crates/chip8-web/roms/. crates/chip8-wasm/web/roms/
python3 -m http.server 8081 -d crates/chip8-wasm/web
```

Bundled ROMs: Pong, Tetris, Space Invaders, Breakout and Tron from the [Chip-8 Program Pack](https://github.com/kripod/chip8-roms), plus [Timendus' chip8-test-suite](https://github.com/Timendus/chip8-test-suite).

## Controls

The player has two keyboard modes:

- **game keys** maps WASD and the arrow keys to each game's controls.
- **classic keypad** uses the original hex layout:

```
CHIP-8 keypad        Keyboard
 1 2 3 C             1 2 3 4
 4 5 6 D             Q W E R
 7 8 9 E             A S D F
 A 0 B F             Z X C V
```

## Project layout

| Crate | Purpose |
|---|---|
| `chip8-core` | Emulator library: CPU, memory, display, timers, keypad |
| `chip8-wasm` | WebAssembly bindings and the static browser player |
| `chip8-web` | Native HTTP/SSE server, canvas rendering, keyboard input |

## Test

```bash
cargo test --workspace
```

The automated suite validates every opcode, all three quirk switches, timers, keypad waits and flow errors. The bundled [Timendus test suite](https://github.com/Timendus/chip8-test-suite) ROMs are for visual conformance checks.

## Deployment

Pushes to `main` build the WASM bundle and deploy `crates/chip8-wasm/web` to GitHub Pages through `.github/workflows/pages.yml`.

## References

- [Cowgod's CHIP-8 reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Guide to writing a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)

## License

MIT OR Apache-2.0
