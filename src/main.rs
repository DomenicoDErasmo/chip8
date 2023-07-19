#[tokio::main]
async fn main() {
    let emulator = chip8::emulator::Emulator::new().await;
    pollster::block_on(emulator.run());
}
