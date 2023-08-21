pub struct InstructionFormat {
    pub first_nibble: u16,
    pub second_nibble: usize,
    pub third_nibble: usize,
    pub fourth_nibble: u16,
    pub nibbles_3_to_4: u8,
    pub nibbles_2_to_4: usize,
}

impl InstructionFormat {
    pub fn new(instruction: &[u8; 2]) -> Self {
        let first_nibble = crate::bit_utils::bit_range_to_num(instruction[0].into(), 4, 8).unwrap();
        let second_nibble =
            crate::bit_utils::bit_range_to_num(instruction[0].into(), 0, 4).unwrap() as usize;
        let third_nibble =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 4, 8).unwrap() as usize;
        let fourth_nibble =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 0, 4).unwrap();
        let nibbles_3_to_4 =
            crate::bit_utils::bit_range_to_num(instruction[1].into(), 0, 8).unwrap() as u8;
        let nibbles_2_to_4 =
            crate::bit_utils::append_number_bits(&[second_nibble as u8, nibbles_3_to_4 as u8])
                as usize;
        Self {
            first_nibble,
            second_nibble,
            third_nibble,
            fourth_nibble,
            nibbles_3_to_4,
            nibbles_2_to_4,
        }
    }
}
