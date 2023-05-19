use crate::stack::Stack;

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
    program_counter: usize,
}

pub struct Instruction {
    instruction: fn() -> (),
    body: RegisterStructure,
}

enum RegisterStructure {
    TWO(TwoRegister),
    ONE(OneRegister),
    NO(NoRegister),
}

struct TwoRegister {
    x_register: u8,
    y_register: u8,
    n_number: u8,
}

struct OneRegister {
    x_register: u8,
    nn_number: u8,
}

struct NoRegister {
    nnn_number: u16,
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
            program_counter: 0,
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

    pub fn fetch(&mut self) -> u16 {
        let instruction_one = self.memory[self.program_counter].clone();
        let instruction_two = self.memory[self.program_counter + 1].clone();
        self.program_counter = self.program_counter + 2;
        let mut result: u16 = 0;

        result = Self::append_bits(result, instruction_one);
        result = Self::append_bits(result, instruction_two);

        result
    }

    fn append_bits(mut appendee: u16, mut appender: u8) -> u16 {
        while appender > 0 {
            appendee = (appendee << 1) | ((appender & 1) as u16);
            appender = appender >> 1;
        }
        appendee
    }
}

// TODO: return Instruction instead of empty
pub fn decode(instruction: u16) -> () {}

// TODO: implement
fn get_value_from_bit_range(instruction: u16, start: u8, end: u8) -> u8 {
    0
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

#[cfg(test)]
mod emulator_tests {
    const ZERO: usize = 0;
    use crate::emulator;

    use super::Emulator;

    #[test]
    fn memory_init() {
        let emulator = emulator::Emulator::new();
        assert_eq!(emulator.memory.len(), emulator::MEMORY_SIZE);
    }

    #[test]
    fn font_init() {
        let emulator = emulator::Emulator::new();
        assert_eq!(emulator.memory[0x50..0x55], [0xF0, 0x90, 0x90, 0x90, 0xF0]);
        assert_eq!(emulator.memory[0x9B..0xA0], [0xF0, 0x80, 0x90, 0x80, 0x80]);
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
        assert_eq!(emulator._delay_timer, ZERO);
        assert_eq!(emulator._sound_timer, ZERO);
    }

    #[test]
    fn program_counter_init() {
        let emulator = emulator::Emulator::new();
        assert_eq!(emulator.program_counter, ZERO);
    }

    #[test]
    fn test_append_bits() {
        let result: u16 = 0b0000_0000_0000_0000;
        let append1: u8 = 0b1010_0101;
        let result = Emulator::append_bits(result, append1);
        assert_eq!(result, 0b1010_0101);
    }
}
