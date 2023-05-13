use chip8::emulator::emulator;

fn main() {
    let mut emulator = emulator::Emulator::new();
    println!("Emulator: {}", &emulator);
    emulator::emulate(&mut emulator);
}
