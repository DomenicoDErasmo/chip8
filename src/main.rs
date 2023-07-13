fn main() {
    pollster::block_on(chip8::window::run());
}
