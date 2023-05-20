use crate::stack::Stack;

const MEMORY_SIZE: usize = 4096;

const WIDTH: u16 = 64;
const HEIGHT: u16 = 32;

#[derive(Debug)]
pub struct Emulator {
    pub memory: [u8; MEMORY_SIZE],
    pub screen: [[bool; WIDTH as usize]; HEIGHT as usize],
    pub stack: Stack<u16>,
    _delay_timer: u8,
    _sound_timer: u8,
    pub program_counter: u16,
    pub registers: [u8; 16],
}

pub struct InstructionSignature {
    _instruction: fn(&mut Emulator, InstructionArguments) -> (),
    _arguments: InstructionArguments,
}

pub struct InstructionArguments {
    _body: RegisterStructure,
    _config: bool,
}

enum RegisterStructure {
    _TWO(TwoRegister),
    _ONE(OneRegister),
    _NO(NoRegister),
}

struct TwoRegister {
    _x_register: u8,
    _y_register: u8,
    _n_number: u8,
}

struct OneRegister {
    _x_register: u8,
    _nn_number: u8,
}

struct NoRegister {
    _nnn_number: u16,
}

impl Emulator {
    pub fn new() -> Emulator {
        let memory: [u8; MEMORY_SIZE] = Self::build_memory();
        Emulator {
            memory,
            screen: [[false; WIDTH as usize]; HEIGHT as usize],
            stack: Stack::<u16>::new(),
            _delay_timer: 0,
            _sound_timer: 0,
            program_counter: 0,
            registers: [0; 16],
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
        let instruction_one = self.memory[self.program_counter as usize].clone();
        let instruction_two = self.memory[(self.program_counter + 1) as usize].clone();
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

    pub fn increment_program_counter(&mut self) {
        self.program_counter = self.program_counter + 2;
    }
}

// TODO: return Instruction instead of empty
pub fn decode(instruction: u16) -> () {
    let _chunk_0 = get_value_from_bit_range(instruction, 0, 3);
    let _chunk_1 = get_value_from_bit_range(instruction, 4, 7);
    let _chunk_2 = get_value_from_bit_range(instruction, 8, 11);
    let _chunk_3 = get_value_from_bit_range(instruction, 12, 15);
}

/// Gets a zero-indexed bit range from instruction
///
/// # Arguments
///
/// * `instruction` - Any 16-bit number
/// * `start` - The rightmost digit
/// * `end` - The leftmost digit
fn get_value_from_bit_range(mut instruction: u16, start: u8, end: u8) -> u16 {
    let mut result_reverse: u16 = 0;
    for _ in 0..start {
        instruction = instruction >> 1;
    }
    for _ in start..=end {
        result_reverse = result_reverse << 1 | (instruction & 1);
        instruction = instruction >> 1;
    }
    let mut result: u16 = 0;
    for _ in start..=end {
        result = result << 1 | (result_reverse & 1);
        result_reverse = result_reverse >> 1;
    }
    result
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
    const ZERO: u8 = 0;
    use crate::emulator;

    use super::{get_value_from_bit_range, Emulator};

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
        assert_eq!(emulator.screen.len() as u16, emulator::HEIGHT);
        assert_eq!(emulator.screen[0].len() as u16, emulator::WIDTH);
    }

    #[test]
    fn stack_init() {
        let emulator = emulator::Emulator::new();
        assert_eq!(emulator.stack.size(), 0);
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
        assert_eq!(emulator.program_counter, ZERO as u16);
    }

    #[test]
    fn test_append_bits() {
        let result: u16 = 0b0000_0000_0000_0000;
        let append1: u8 = 0b1010_0101;
        let result = Emulator::append_bits(result, append1);
        assert_eq!(result, 0b1010_0101);
    }

    #[test]
    fn test_get_value_from_bit_range() {
        let instruction: u16 = 0b1000_0000_0000_0000;
        let result = get_value_from_bit_range(instruction, 12, 15);
        assert_eq!(result, 0b0000_0000_0000_1000);

        let instruction: u16 = 0b0000_1101_0101_0000;
        let result = get_value_from_bit_range(instruction, 4, 11);
        assert_eq!(result, 0b0000_0000_1101_0101);

        let instruction: u16 = 0b0000_0110_1011_0111;
        let result = get_value_from_bit_range(instruction, 0, 11);
        assert_eq!(result, 0b0000_0110_1011_0111);
    }
}
