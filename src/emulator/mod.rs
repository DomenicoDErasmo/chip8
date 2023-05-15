pub mod emulator {
    use crate::stack::stack::Stack;
    use std::{
        thread::sleep,
        time::{Duration, SystemTime},
    };

    const MEMORY_SIZE: usize = 4096;

    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

    #[derive(Debug)]
    pub struct Emulator {
        memory: [u8; MEMORY_SIZE],
        screen: [[bool; WIDTH]; HEIGHT],
        _stack: Stack<u16>,
        _delay_timer: usize,
        _sound_timer: usize,
    }

    impl Emulator {
        pub fn new() -> Emulator {
            let memory: [u8; MEMORY_SIZE] = Self::build_memory();
            Emulator {
                memory,
                screen: [[false; WIDTH]; HEIGHT],
                _stack: Stack::<u16>::new(),
                _delay_timer: 0,
                _sound_timer: 0,
            }
        }

        fn build_memory() -> [u8; MEMORY_SIZE] {
            let mut memory = [0; MEMORY_SIZE];
            Self::load_font(&mut memory);
            memory
        }

        fn load_font(memory: &mut [u8; MEMORY_SIZE]) {
            // 0 through F
            memory[0x50..0x55].copy_from_slice(&[0xF0, 0x90, 0x90, 0x90, 0xF0]);
            memory[0x55..0x5A].copy_from_slice(&[0x20, 0x60, 0x20, 0x20, 0x70]);
            memory[0x5A..0x5F].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x80, 0xF0]);
            memory[0x5F..0x64].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x10, 0xF0]);
            memory[0x64..0x69].copy_from_slice(&[0x90, 0x90, 0xF0, 0x10, 0x10]);
            memory[0x69..0x6E].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x10, 0xF0]);
            memory[0x6E..0x73].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x90, 0xF0]);
            memory[0x73..0x78].copy_from_slice(&[0xF0, 0x10, 0x20, 0x40, 0x40]);
            memory[0x78..0x7D].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0xF0]);
            memory[0x7D..0x82].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x10, 0xF0]);
            memory[0x82..0x87].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0x90]);
            memory[0x87..0x8C].copy_from_slice(&[0xE0, 0x90, 0xE0, 0x90, 0xE0]);
            memory[0x8C..0x91].copy_from_slice(&[0xF0, 0x80, 0x80, 0x80, 0xF0]);
            memory[0x91..0x96].copy_from_slice(&[0xE0, 0x90, 0x90, 0x90, 0xF0]);
            memory[0x96..0x9B].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0xF0]);
            memory[0x9B..0xA0].copy_from_slice(&[0xF0, 0x80, 0x90, 0x80, 0x80]);
        }
    }

    impl std::fmt::Display for Emulator {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            let final_output = &(format!(
                "Memory: [size: {}, height: {}, width: {}]",
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

        const HERTZ: u32 = 60;
        let mut counter = 0;

        while running {
            let mut current_time = current_timestamp();

            // end of processing for this visual frame
            let time_delta = match u32::try_from(current_time - last_time) {
                Ok(time) => time,
                Err(_) => panic!("Conversion error!"),
            };

            if counter >= HERTZ {
                println!("A second has passed");
                counter = 0;
            }
            counter = counter + 1;

            sleep(Duration::new(0, nanoseconds_per_cycle(HERTZ) - time_delta));

            current_time = current_timestamp();
            last_time = current_time;
        }
    }

    fn nanoseconds_per_cycle(frequency: u32) -> u32 {
        // taking duration of one cycle and multiplying by 1,000,000 to get the desired precision
        (1_000_000.0 / frequency as f32).round() as u32
    }

    #[cfg(test)]
    mod emulator_tests {
        const TIMER_INIT_SIZE: usize = 0;
        use crate::emulator::emulator;

        #[test]
        fn memory_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator.memory.len(), emulator::MEMORY_SIZE);
        }

        #[test]
        fn font_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator.memory[0x50..0x55], [0xF0, 0x90, 0x90, 0x90, 0xF0]); // 0
            assert_eq!(emulator.memory[0x9B..0xA0], [0xF0, 0x80, 0x90, 0x80, 0x80]);
            // F
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
            assert_eq!(emulator._stack.size(), 0);
        }

        #[test]
        fn timer_init() {
            let emulator = emulator::Emulator::new();
            assert_eq!(emulator._delay_timer, TIMER_INIT_SIZE);
            assert_eq!(emulator._sound_timer, TIMER_INIT_SIZE);
        }
    }
}
