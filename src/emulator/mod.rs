use std::num::IntErrorKind;

use crate::{function, stack::Stack};

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

pub struct InstructionSignature {
    function: fn(&mut Emulator, InstructionArguments) -> (),
    arguments: InstructionArguments,
}

impl InstructionSignature {
    pub fn execute(&self, emulator: &mut Emulator) {
        (self.function)(emulator, self.arguments.clone());
    }
}

#[derive(Clone)]
pub struct InstructionArguments {
    _body: RegisterStructure,
    _config: bool,
}

#[derive(Clone)]
enum RegisterStructure {
    TWO(TwoRegister),
    ONE(OneRegister),
    NO(NoRegister),
}

#[derive(Clone)]
struct TwoRegister {
    _x_register: u16,
    _y_register: u16,
    _n_number: u16,
}

#[derive(Clone)]
struct OneRegister {
    _x_register: u16,
    _nn_number: u16,
}

#[derive(Clone)]
struct NoRegister {
    _nnn_number: u16,
}

/// Decodes the instruction and turns it into one of the below functions. Functions can consist of the following properties:
/// - First nibble: kind of instruction
///     - Either
///         - Second nibble (X): a register
///             - Either
///                 - Third nibble (Y): a register
///                 - Fourth nibble (N): a number argument
///             - OR
///                 - Third and fourth nibbles (NN): a number argument
///     - OR
///         - Second, third, and fourth nibbles (NNN): a number argument
///
/// These are typically expressed as 0XYN, 0XNN, or 0NNN, where the first digit can vary from 0 to F
///
/// # Arguments
///
/// instruction: A 16-bit number
///
/// # Errors
///
/// IntErrorKind::InvalidDigit: We passed a wrongly-formatted instruction
pub fn decode(instruction: u16) -> Result<InstructionSignature, IntErrorKind> {
    let nibble_0 = get_value_from_bit_range(instruction, 0, 3);
    let nibble_1 = get_value_from_bit_range(instruction, 4, 7);
    let nibble_2 = get_value_from_bit_range(instruction, 8, 11);
    let nibble_3 = get_value_from_bit_range(instruction, 12, 15);

    let nibbles_01 = get_value_from_bit_range(instruction, 0, 7);
    let nibbles_012 = get_value_from_bit_range(instruction, 0, 11);

    match nibble_3 {
        0x0 => match instruction {
            0x0 => Ok(InstructionSignature {
                function: function::clear_screen_00e0,
                arguments: InstructionArguments {
                    _body: RegisterStructure::NO(NoRegister { _nnn_number: 0 }),
                    _config: false,
                },
            }),
            0xE => Ok(InstructionSignature {
                function: function::subroutine_return_00ee,
                arguments: InstructionArguments {
                    _body: RegisterStructure::NO(NoRegister { _nnn_number: 0 }),
                    _config: false,
                },
            }),
            _ => Err(IntErrorKind::InvalidDigit),
        },
        0x1 => Ok(InstructionSignature {
            function: function::jump_1nnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::NO(NoRegister {
                    _nnn_number: nibbles_012,
                }),
                _config: false,
            },
        }),
        0x2 => Ok(InstructionSignature {
            function: function::subroutine_2nnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::NO(NoRegister {
                    _nnn_number: nibbles_012,
                }),
                _config: false,
            },
        }),
        0x3 => Ok(InstructionSignature {
            function: function::skip_if_equal_3xnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::ONE(OneRegister {
                    _x_register: nibble_2,
                    _nn_number: nibbles_01,
                }),
                _config: false,
            },
        }),
        0x4 => Ok(InstructionSignature {
            function: function::skip_if_not_equal_4xnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::ONE(OneRegister {
                    _x_register: nibble_2,
                    _nn_number: nibbles_01,
                }),
                _config: false,
            },
        }),
        0x5 => match nibble_0 {
            0x0 => Ok(InstructionSignature {
                function: function::skip_if_registers_equal_5xy0,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: 0,
                    }),
                    _config: false,
                },
            }),
            _ => Err(IntErrorKind::InvalidDigit),
        },
        0x6 => Ok(InstructionSignature {
            function: function::set_register_to_6xnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::ONE(OneRegister {
                    _x_register: nibble_2,
                    _nn_number: nibbles_01,
                }),
                _config: false,
            },
        }),
        0x7 => Ok(InstructionSignature {
            function: function::add_num_to_register_7xnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::ONE(OneRegister {
                    _x_register: nibble_2,
                    _nn_number: nibbles_01,
                }),
                _config: false,
            },
        }),
        0x8 => match nibble_0 {
            0x0 => Ok(InstructionSignature {
                function: function::set_one_register_to_another_8xy0,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0x1 => Ok(InstructionSignature {
                function: function::binary_or_registers_8xy1,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0x2 => Ok(InstructionSignature {
                function: function::binary_and_registers_8xy2,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0x3 => Ok(InstructionSignature {
                function: function::binary_xor_register_8xy3,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0x4 => Ok(InstructionSignature {
                function: function::add_register_to_register_8xy4,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0x5 => Ok(InstructionSignature {
                function: function::subtract_right_from_left_8xy5,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0x6 => Ok(InstructionSignature {
                function: function::shift_right_8xy6,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0x7 => Ok(InstructionSignature {
                function: function::subtract_left_from_right_8xy7,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            0xE => Ok(InstructionSignature {
                function: function::shift_left_8xye,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: nibble_0,
                    }),
                    _config: false,
                },
            }),
            _ => Err(IntErrorKind::InvalidDigit),
        },
        0x9 => match nibble_0 {
            0x0 => Ok(InstructionSignature {
                function: function::skip_if_registers_not_equal_9xy0,
                arguments: InstructionArguments {
                    _body: RegisterStructure::TWO(TwoRegister {
                        _x_register: nibble_2,
                        _y_register: nibble_1,
                        _n_number: 0,
                    }),
                    _config: false,
                },
            }),
            _ => Err(IntErrorKind::InvalidDigit),
        },
        0xA => Ok(InstructionSignature {
            function: function::set_index_annn,
            arguments: InstructionArguments {
                _body: RegisterStructure::NO(NoRegister {
                    _nnn_number: nibbles_012,
                }),
                _config: false,
            },
        }),
        0xB => Ok(InstructionSignature {
            function: function::jump_with_offset_bnnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::NO(NoRegister {
                    _nnn_number: nibbles_012,
                }),
                _config: false,
            },
        }),
        0xC => Ok(InstructionSignature {
            function: function::random_cxnn,
            arguments: InstructionArguments {
                _body: RegisterStructure::ONE(OneRegister {
                    _x_register: nibble_2,
                    _nn_number: nibbles_01,
                }),
                _config: false,
            },
        }),
        0xD => Ok(InstructionSignature {
            function: function::display_dxyn,
            arguments: InstructionArguments {
                _body: RegisterStructure::TWO(TwoRegister {
                    _x_register: nibble_2,
                    _y_register: nibble_1,
                    _n_number: nibble_0,
                }),
                _config: false,
            },
        }),
        0xE => match nibbles_01 {
            0x9E => Ok(InstructionSignature {
                function: function::skip_if_pressed_ex9e,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0xA1 => Ok(InstructionSignature {
                function: function::skip_if_not_pressed_exa1,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            _ => Err(IntErrorKind::InvalidDigit),
        },
        0xF => match nibbles_01 {
            0x07 => Ok(InstructionSignature {
                function: function::set_to_delay_timer_fx07,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x15 => Ok(InstructionSignature {
                function: function::set_delay_timer_to_fx15,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x18 => Ok(InstructionSignature {
                function: function::set_sound_timer_to_fx18,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x1E => Ok(InstructionSignature {
                function: function::add_to_index_fx1e,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x0A => Ok(InstructionSignature {
                function: function::get_key_fx0a,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x29 => Ok(InstructionSignature {
                function: function::set_register_to_character_fx29,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x33 => Ok(InstructionSignature {
                function: function::binary_coded_decimal_conversion_fx33,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x55 => Ok(InstructionSignature {
                function: function::store_to_memory_fx55,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            0x65 => Ok(InstructionSignature {
                function: function::load_from_memory_fx56,
                arguments: InstructionArguments {
                    _body: RegisterStructure::ONE(OneRegister {
                        _x_register: nibble_2,
                        _nn_number: 0,
                    }),
                    _config: false,
                },
            }),
            _ => Err(IntErrorKind::InvalidDigit),
        },
        _ => Err(IntErrorKind::InvalidDigit),
    }
}

/// Gets a zero-indexed, right-to-left bit range from instruction
/// TODO: can we do this more elegantly with a bitmask?
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
