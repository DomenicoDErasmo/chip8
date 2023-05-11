pub struct Emulator {
    memory: [u8; 4096],
}

// TODO: test memory construction works, add font reference, better memory allocation?
impl Emulator {
    pub fn new() -> Emulator {
        let memory = Self::build_memory();
        Emulator { 
            memory,
        }
    }

    fn build_memory() -> [u8; 4096] {
        let mut memory = [0; 4096];
        Self::load_font(&mut memory);
        memory
    }

    fn load_font(memory: &mut [u8; 4096]) {
        memory[79..84].copy_from_slice(&[0xF0, 0x90, 0x90, 0x90, 0xF0]); // 0
        memory[85..89].copy_from_slice(&[0x20, 0x60, 0x20, 0x20, 0x70]); // 1
        memory[90..94].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x80, 0xF0]); // 2
        memory[95..99].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x10, 0xF0]); // 3
        memory[100..104].copy_from_slice(&[0x90, 0x90, 0xF0, 0x10, 0x10]); // 4
        memory[105..109].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x10, 0xF0]); // 5
        memory[110..114].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x90, 0xF0]); // 6
        memory[115..119].copy_from_slice(&[0xF0, 0x10, 0x20, 0x40, 0x40]); // 7
        memory[120..124].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0xF0]); // 8
        memory[125..129].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x10, 0xF0]); // 9
        memory[130..134].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0x90]); // A
        memory[135..139].copy_from_slice(&[0xE0, 0x90, 0xE0, 0x90, 0xE0]); // B
        memory[140..144].copy_from_slice(&[0xF0, 0x80, 0x80, 0x80, 0xF0]); // C
        memory[145..149].copy_from_slice(&[0xE0, 0x90, 0x90, 0x90, 0xF0]); // D
        memory[150..154].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0xF0]); // E
        memory[155..159].copy_from_slice(&[0xF0, 0x80, 0x90, 0x80, 0x80]); // F
    }

    pub fn hello(self) {
        println!("Hello world! I store {} bytes.", self.memory.len());
    } 
}