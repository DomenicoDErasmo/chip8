#[tokio::main]
async fn main() {
    let file_path = "/roms/IBM Logo.ch8";
    let emulator = chip8::emulator::Emulator::new(file_path.into()).await;
    pollster::block_on(emulator.run());
}
