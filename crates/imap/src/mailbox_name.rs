pub fn decode_mailbox_name(name: &str) -> String {
    let mut decoded = String::with_capacity(name.len());
    let mut rest = name;
    while let Some(start) = rest.find('&') {
        decoded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('-') else {
            decoded.push_str(&rest[start..]);
            return decoded;
        };
        let encoded = &after[..end];
        if encoded.is_empty() {
            decoded.push('&');
        } else {
            match decode_utf16_base64(encoded) {
                Some(text) => decoded.push_str(&text),
                None => decoded.push_str(&rest[start..start + end + 2]),
            }
        }
        rest = &after[end + 1..];
    }
    decoded.push_str(rest);
    decoded
}

fn decode_utf16_base64(encoded: &str) -> Option<String> {
    let mut bits = 0u32;
    let mut bit_count = 0;
    let mut bytes = Vec::with_capacity(encoded.len());
    for c in encoded.bytes() {
        let value = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b',' => 63,
            _ => return None,
        };
        bits = (bits << 6) | u32::from(value);
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            bytes.push((bits >> bit_count) as u8);
            bits &= (1 << bit_count) - 1;
        }
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16(&units).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_modified_utf7() {
        assert_eq!(decode_mailbox_name("INBOX"), "INBOX");
        assert_eq!(decode_mailbox_name("Entw&APw-rfe"), "Entwürfe");
        assert_eq!(
            decode_mailbox_name("[Gmail]/Lixeira &- Spam"),
            "[Gmail]/Lixeira & Spam"
        );
        assert_eq!(decode_mailbox_name("&ZeVnLIqe-"), "日本語");
        assert_eq!(decode_mailbox_name("broken&abc"), "broken&abc");
    }
}
