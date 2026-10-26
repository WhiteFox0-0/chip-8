use chip8_core::cpu::CPU;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const INDEX: &str = include_str!("index.html");

struct Game {
    id: &'static str,
    name: &'static str,
    group: &'static str,
    rom: &'static [u8],
}

const GAMES: &[Game] = &[
    Game { id: "pong", name: "Pong", group: "games", rom: include_bytes!("../roms/pong.ch8") },
    Game { id: "tetris", name: "Tetris", group: "games", rom: include_bytes!("../roms/tetris.ch8") },
    Game { id: "invaders", name: "Space Invaders", group: "games", rom: include_bytes!("../roms/invaders.ch8") },
    Game { id: "breakout", name: "Breakout", group: "games", rom: include_bytes!("../roms/breakout.ch8") },
    Game { id: "tron", name: "Tron", group: "games", rom: include_bytes!("../roms/tron.ch8") },
    Game { id: "logo", name: "1. CHIP-8 logo", group: "tests", rom: include_bytes!("../roms/tests/1-chip8-logo.ch8") },
    Game { id: "ibm", name: "2. IBM logo", group: "tests", rom: include_bytes!("../roms/tests/2-ibm-logo.ch8") },
    Game { id: "corax", name: "3. Corax+", group: "tests", rom: include_bytes!("../roms/tests/3-corax+.ch8") },
    Game { id: "flags", name: "4. Flags", group: "tests", rom: include_bytes!("../roms/tests/4-flags.ch8") },
    Game { id: "quirks", name: "5. Quirks", group: "tests", rom: include_bytes!("../roms/tests/5-quirks.ch8") },
    Game { id: "keypad", name: "6. Keypad", group: "tests", rom: include_bytes!("../roms/tests/6-keypad.ch8") },
    Game { id: "beep", name: "7. Beep", group: "tests", rom: include_bytes!("../roms/tests/7-beep.ch8") },
    Game { id: "scrolling", name: "8. Scrolling", group: "tests", rom: include_bytes!("../roms/tests/8-scrolling.ch8") },
];

fn game(id: &str) -> Option<&'static Game> {
    GAMES.iter().find(|g| g.id == id)
}

#[derive(Default)]
struct Shared {
    keys: [bool; 16],
}

fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);
    let shared = Arc::new(Mutex::new(Shared::default()));
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("could not bind port");

    println!("chip8 web running at http://127.0.0.1:{port}");
    for stream in listener.incoming().flatten() {
        let shared = Arc::clone(&shared);
        thread::spawn(move || handle(stream, shared));
    }
}

fn handle(stream: TcpStream, shared: Arc<Mutex<Shared>>) {
    let mut reader = match stream.try_clone() {
        Ok(s) => BufReader::new(s),
        Err(_) => return,
    };

    let mut request = String::new();
    if reader.read_line(&mut request).unwrap_or(0) == 0 {
        return;
    }
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let full = parts.next().unwrap_or("/");
    let (path, query) = full.split_once('?').unwrap_or((full, ""));

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }
    if content_length > 0 {
        let mut discard = vec![0u8; content_length];
        let _ = reader.read_exact(&mut discard);
    }

    match (method.as_str(), path) {
        ("GET", "/") => respond(&stream, "200 OK", "text/html; charset=utf-8", INDEX.as_bytes()),
        ("GET", "/api/games") => {
            respond(&stream, "200 OK", "application/json", games_json().as_bytes());
        }
        ("GET", "/events") => match query_param(query, "game").and_then(game) {
            Some(g) => stream_frames(stream, shared, g),
            None => respond(&stream, "404 Not Found", "text/plain", b"unknown game"),
        },
        ("GET", p) if p.starts_with("/preview/") && p.ends_with(".svg") => {
            let id = &p["/preview/".len()..p.len() - 4];
            match game(id) {
                Some(g) => respond(&stream, "200 OK", "image/svg+xml", preview_svg(g).as_bytes()),
                None => respond(&stream, "404 Not Found", "text/plain", b"unknown game"),
            }
        }
        ("POST", p) if p.starts_with("/key/") => {
            let mut it = p["/key/".len()..].split('/');
            let key = it.next().and_then(|h| u8::from_str_radix(h, 16).ok());
            let down = it.next() == Some("down");
            if let Some(key) = key.filter(|k| *k < 16) {
                shared.lock().unwrap().keys[key as usize] = down;
            }
            respond(&stream, "200 OK", "text/plain", b"ok");
        }
        _ => respond(&stream, "404 Not Found", "text/plain", b"not found"),
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then_some(v)
    })
}

fn games_json() -> String {
    let mut out = String::from("[");
    for (i, g) in GAMES.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"id\":\"{}\",\"name\":\"{}\",\"group\":\"{}\"}}",
            g.id, g.name, g.group
        ));
    }
    out.push(']');
    out
}

fn preview_svg(g: &Game) -> String {
    let mut cpu = CPU::new();
    let _ = cpu.load_rom(g.rom);
    for _ in 0..150 {
        for _ in 0..12 {
            cpu.step();
        }
        cpu.tick_timers();
    }

    let mut svg = String::with_capacity(512 + 2048 * 40);
    svg.push_str(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 64 32\" shape-rendering=\"crispEdges\">\
         <rect width=\"64\" height=\"32\" fill=\"#050607\"/><g fill=\"#39ff14\">",
    );
    for (p, &on) in cpu.framebuffer().iter().enumerate() {
        if on {
            svg.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"1\" height=\"1\"/>",
                p % 64,
                p / 64
            ));
        }
    }
    svg.push_str("</g></svg>");
    svg
}

fn respond(mut stream: &TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn stream_frames(mut stream: TcpStream, shared: Arc<Mutex<Shared>>, g: &'static Game) {
    let _ = stream.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
    );
    let _ = stream.flush();

    let mut cpu = CPU::new();
    let _ = cpu.load_rom(g.rom);

    loop {
        let keys = shared.lock().unwrap().keys;
        for (key, pressed) in keys.iter().enumerate() {
            cpu.set_key(key as u8, *pressed);
        }

        for _ in 0..12 {
            cpu.step();
        }
        cpu.tick_timers();

        let mut frame = String::with_capacity(257);
        for chunk in cpu.framebuffer().chunks(8) {
            let byte = chunk.iter().fold(0u8, |b, &on| (b << 1) | on as u8);
            frame.push_str(&format!("{byte:02x}"));
        }
        frame.push('|');
        frame.push(if cpu.sound_active() { '1' } else { '0' });

        if write!(stream, "data: {frame}\n\n").is_err() || stream.flush().is_err() {
            break;
        }
        thread::sleep(Duration::from_millis(16));
    }
}
