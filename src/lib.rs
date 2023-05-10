pub struct Emulator {
    memory: [i8; 4096],
}

impl Emulator {
    pub fn new() -> Emulator{
        Emulator { 
            memory: [0; 4096], 
        }
    }

    pub fn hello(self) {
        println!("Hello world! I store {} bytes.", self.memory.len());
    } 
}