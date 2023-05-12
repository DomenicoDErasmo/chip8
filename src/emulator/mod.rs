pub mod emulator {
    use std::{time::{SystemTime, Duration}, thread::sleep};
    use crate::stack::stack::Stack;

    const MEMORY_SIZE: usize = 4096;

    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

    const TIMER_INIT_SIZE: usize = 0;

    #[derive(Debug)]
    pub struct Emulator {
        memory: [u8; MEMORY_SIZE],
        screen: [[bool; WIDTH]; HEIGHT],
        stack: Stack<u16>,
        delay_timer: usize,
        sound_timer: usize,
    }

    impl Emulator {
        pub fn new() -> Emulator {
            let memory: [u8; MEMORY_SIZE] = Self::build_memory();
            Emulator { 
                memory,
                screen: [[false; WIDTH]; HEIGHT],
                stack: Stack::<u16>::new(),
                delay_timer: 0,
                sound_timer: 0,
            }
        }

        fn build_memory() -> [u8; MEMORY_SIZE] {
            let mut memory = [0; MEMORY_SIZE];
            Self::load_font(&mut memory);
            memory
        }

        fn load_font(memory: &mut [u8; MEMORY_SIZE]) {
            memory[0x50..0x55].copy_from_slice(&[0xF0, 0x90, 0x90, 0x90, 0xF0]); // 0
            memory[0x55..0x5A].copy_from_slice(&[0x20, 0x60, 0x20, 0x20, 0x70]); // 1
            memory[0x5A..0x5F].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x80, 0xF0]); // 2
            memory[0x5F..0x64].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x10, 0xF0]); // 3
            memory[0x64..0x69].copy_from_slice(&[0x90, 0x90, 0xF0, 0x10, 0x10]); // 4
            memory[0x69..0x6E].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x10, 0xF0]); // 5
            memory[0x6E..0x73].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x90, 0xF0]); // 6
            memory[0x73..0x78].copy_from_slice(&[0xF0, 0x10, 0x20, 0x40, 0x40]); // 7
            memory[0x78..0x7D].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0xF0]); // 8
            memory[0x7D..0x82].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x10, 0xF0]); // 9
            memory[0x82..0x87].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0x90]); // A
            memory[0x87..0x8C].copy_from_slice(&[0xE0, 0x90, 0xE0, 0x90, 0xE0]); // B
            memory[0x8C..0x91].copy_from_slice(&[0xF0, 0x80, 0x80, 0x80, 0xF0]); // C
            memory[0x91..0x96].copy_from_slice(&[0xE0, 0x90, 0x90, 0x90, 0xF0]); // D
            memory[0x96..0x9B].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0xF0]); // E
            memory[0x9B..0xA0].copy_from_slice(&[0xF0, 0x80, 0x90, 0x80, 0x80]); // F
        }

    }

    impl std::fmt::Display for Emulator {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            let final_output = &(format!("Memory: [size: {}, height: {}, width: {}]", 
                self.memory.len().to_owned(),
                self.screen.len().to_owned(),
                self.screen[0].len().to_owned(),
            ));
            write!(f, "{}", final_output)
        }
    }
    
    fn current_timestamp() -> u128 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => time.as_millis(),
            Err(_) => panic!("System time before UNIX EPOCH!"),
        }
    }

    pub fn emulate(_emulator: &mut Emulator) {
        let running = true;
        let mut last_time = current_timestamp();

        while running {
            let mut current_time = current_timestamp();
            let time_delta = match u32::try_from(current_time - last_time) {
                Ok(time) => time,
                Err(_) => panic!("Conversion error!"),
            };

            sleep(Duration::new(0, 1_000_000 * (16 - time_delta)));
            current_time = current_timestamp();
            last_time = current_time;
        }
    }
    
    #[cfg(test)]
    mod emulator_tests {
        use crate::emulator::emulator::{self, TIMER_INIT_SIZE};

        #[test]
        fn memory_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator.memory.len(), emulator::MEMORY_SIZE);
        }

        #[test]
        fn font_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator.memory[0x50..0x55], [0xF0, 0x90, 0x90, 0x90, 0xF0]); // 0
            assert_eq!(emulator.memory[0x9B..0xA0], [0xF0, 0x80, 0x90, 0x80, 0x80]); // F
        }
        
        #[test]
        fn screen_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator.screen.len(), emulator::HEIGHT);
            assert_eq!(emulator.screen[0].len(), emulator::WIDTH);
        }

        #[test]
        fn stack_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator.stack.size(), 0);
        }

        #[test]
        fn timer_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator.delay_timer, TIMER_INIT_SIZE);
            assert_eq!(emulator.sound_timer, TIMER_INIT_SIZE);
        }

    }
}
