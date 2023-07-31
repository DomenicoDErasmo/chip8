pub fn append_number_bits(nums: &[u8; 2]) -> u16 {
    let [left_num, right_num]: [u8; 2] = *nums;
    let mut result: u16 = bit_range_to_num(left_num.into(), 0, 8).unwrap() << 8;
    result = result | bit_range_to_num(right_num.into(), 0, 8).unwrap();
    result
}

pub fn bit_range_to_num(num: u16, start: usize, end: usize) -> Option<u16> {
    if (start > end) | (start >= 16) | (end > 16) {
        return None;
    }
    let mut result = 0;
    for i in start..end {
        let ith_bit = num & (1 << i);
        result = result | ith_bit;
    }
    Some(result >> start)
}

#[test]
fn test_append_number_bits() {
    let nums = [0b10000101, 0b01101100];
    assert_eq!(append_number_bits(&nums), 0b1000010101101100);
    let nums = [0b10000101, 0b11101101];
    assert_eq!(append_number_bits(&nums), 0b1000010111101101);
}

#[test]
fn test_bit_range_to_num() {
    assert_eq!(bit_range_to_num(0b01101101, 0, 4), Some(0b1101));
    assert_eq!(bit_range_to_num(0b01101101, 4, 8), Some(0b0110));
    assert_eq!(bit_range_to_num(0b01101101, 5, 2), None);
    assert_eq!(bit_range_to_num(0b01101101, 16, 20), None);
    assert_eq!(bit_range_to_num(0b01101101, 14, 17), None);
}
