pub fn to_sortable_base_16(data: &[u8]) -> String {
    let mut result = String::new();

    for byte in data {
        let upper = (('a' as u8) + (byte >> 4)) as char;
        let lower = (('a' as u8) + (byte & 0b00001111)) as char;

        result.push(upper);
        result.push(lower);
    }

    return result;
}
