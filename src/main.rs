use std::num::IntErrorKind;

use chip8::emulator;
use chip8::window;

fn main() -> Result<(), IntErrorKind> {
    let emulator = emulator::Emulator::new();
    let mut window = window::MyWindow::new(emulator);
    window.run()?;
    Ok(())
}
