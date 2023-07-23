pub fn get_u16_bit_range_as_number(num: u16, start: usize, end: usize) -> Option<u16> {
    if (start >= end) | (start > 16) | (end > 16) {
        return None;
    }

    let mut result = 0;
    for i in start..end {
        result = set_bit_from_other_number(num, i, result, i - start);
    }

    Some(result)
}

pub fn set_bit_from_other_number(
    source: u16,
    source_bit: usize,
    target: u16,
    target_bit: usize,
) -> u16 {
    let nth_bit = get_u128_bit_at(source.into(), source_bit).unwrap();
    set_u16_bit_at(target, target_bit, nth_bit).unwrap()
}

pub fn get_u128_bit_at(num: u128, n: usize) -> Option<bool> {
    if n < 128 {
        Some(num & (1 << n) != 0)
    } else {
        None
    }
}

pub fn set_u16_bit_at(target: u16, n: usize, value: bool) -> Option<u16> {
    if n < 16 {
        Some(target | ((value as u16) << n))
    } else {
        None
    }
}

#[test]
fn test_get_u8_bit_at() {
    assert_eq!(get_u128_bit_at(0b10001010, 0), Some(false));
    assert_eq!(get_u128_bit_at(0b10001010, 1), Some(true));
    assert_eq!(get_u128_bit_at(0b10001010, 7), Some(true));
    assert_eq!(get_u128_bit_at(0b10001010, 228), None);
}

#[test]
fn test_set_u8_bit_at() {
    assert_eq!(get_u128_bit_at(0b10001010, 0), Some(false));
    assert_eq!(get_u128_bit_at(0b10001010, 1), Some(true));
    assert_eq!(get_u128_bit_at(0b10001010, 7), Some(true));
    assert_eq!(get_u128_bit_at(0b10001010, 228), None);
}

#[test]
fn test_get_u16_range_as_number() {
    assert_eq!(get_u16_bit_range_as_number(0xABCD, 0, 4), Some(0xD));
    assert_eq!(get_u16_bit_range_as_number(0xABCD, 12, 16), Some(0xA));
    assert_eq!(get_u16_bit_range_as_number(0xABCD, 12, 17), None);
    assert_eq!(get_u16_bit_range_as_number(0xABCD, 17, 20), None);
    assert_eq!(get_u16_bit_range_as_number(0xABCD, 2, 0), None);
    assert_eq!(get_u16_bit_range_as_number(0xABCD, 2, 2), None);
}

#[test]
fn test_set_bit_from_other_number() {
    assert_eq!(
        set_bit_from_other_number(0b1001101, 2, 0b110000, 3),
        0b111000
    );
}
