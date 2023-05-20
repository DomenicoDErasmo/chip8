use crate::emulator::{self, Emulator};

pub fn _clear_screen_00e0(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {
}

pub fn _jump_1nnn(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _subroutine_2nnn(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _subroutine_00ee_return(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _skip_if_equal_3xnn(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _skip_if_not_equal_4xnn(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _skip_if_registers_equal_5xy0(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _skip_if_registers_equal_9xy0(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _set_register_to_6xnn(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _add_num_to_register_7xnn(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _set_one_register_to_another_8xy0(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _binary_or_registers_8xy1(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _binary_and_registers_8xy2(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _binary_xor_register_8xy3(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _add_register_to_register_8xy4(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _subtract_right_from_left_8xy5(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _subtract_left_from_right_8xy7(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _shift_right_8xy6(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _shift_left_8xye(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _set_index_annn(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _jump_with_offset_bnnn(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _random_cxnn(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _display_dxyn(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _skip_if_pressed_ex9e(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _skip_if_not_pressed_exa1(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _set_to_delay_timer_fx07(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _set_delay_timer_to_fx15(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _set_sound_timer_to_fx18(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _add_to_index_fx1e(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {
}

pub fn _get_key_fx0a(_emulator: &mut Emulator, _instructions: emulator::InstructionSignature) {}

pub fn _set_register_to_character_fx29(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _binary_coded_decimal_conversion_fx33(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _store_to_memory_fx55(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}

pub fn _load_from_memory_fx56(
    _emulator: &mut Emulator,
    _instructions: emulator::InstructionSignature,
) {
}
