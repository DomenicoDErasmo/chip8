#[tokio::main]
async fn main() {
    let file_path = "./roms/test_opcode.ch8";
    let emulator = chip8::emulator::Emulator::new(Some(file_path), false, false).await;
    pollster::block_on(emulator.run());
}
