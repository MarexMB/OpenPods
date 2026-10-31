/// Redact conventional Bluetooth addresses and BlueZ device path fragments.
/// This is not anonymization: never collect protocol dumps or pairing material.
pub fn redact(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if offset + 17 <= bytes.len() {
            let part = &bytes[offset..offset + 17];
            let separator = part[2];
            if matches!(separator, b':' | b'_') && part.iter().enumerate().all(|(i, b)| {
                if i % 3 == 2 { *b == separator } else { b.is_ascii_hexdigit() }
            }) {
                output.push_str("[device-id]");
                offset += 17;
                continue;
            }
        }
        let character = input[offset..].chars().next().expect("valid UTF-8 boundary");
        if !character.is_control() || matches!(character, '\n' | '\t') { output.push(character); }
        offset += character.len_utf8();
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redact_addresses_and_keep_unicode() {
        assert_eq!(redact("Đ AirPods AA:bb:00:11:22:33 /dev_AA_BB_00_11_22_33\u{1b}"), "Đ AirPods [device-id] /dev_[device-id]");
        assert_eq!(redact("90%"), "90%");
    }
}
