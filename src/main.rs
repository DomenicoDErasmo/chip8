use chip8::emulator;
use chip8::window;

fn main() {
    let emulator = emulator::Emulator::new();
    let window = window::MyWindow::new(emulator);
    window.run();
}
