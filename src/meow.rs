// Each byte encodes as 4 cat sounds (base-4, 2 bits per sound, MSB first):
//   00 = nya   01 = mew   10 = purr   11 = meow
// example: 0b10_01_00_11 = purr mew nya meow

const SOUNDS: [&str; 4] = ["nya", "mew", "purr", "meow"];
const SOUNDS_PER_LINE: usize = 16; // 4 bytes per line — readable width

pub fn encode(data: &[u8]) -> String {
    let words: Vec<&str> = data
        .iter()
        .flat_map(|&byte| {
            [
                SOUNDS[((byte >> 6) & 0b11) as usize],
                SOUNDS[((byte >> 4) & 0b11) as usize],
                SOUNDS[((byte >> 2) & 0b11) as usize],
                SOUNDS[(byte & 0b11) as usize],
            ]
        })
        .collect();

    words
        .chunks(SOUNDS_PER_LINE)
        .map(|chunk| chunk.join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn decode(text: &str) -> anyhow::Result<Vec<u8>> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if !tokens.len().is_multiple_of(4) {
        anyhow::bail!(
            "meow sequence length {} not divisible by 4 >:3",
            tokens.len()
        );
    }
    tokens
        .chunks(4)
        .map(|chunk| {
            let mut byte = 0u8;
            for (i, &tok) in chunk.iter().enumerate() {
                let idx = SOUNDS
                    .iter()
                    .position(|&s| s == tok)
                    .ok_or_else(|| anyhow::anyhow!("unknown sound '{}' >:3", tok))?;
                byte |= (idx as u8) << (6 - i * 2);
            }
            Ok(byte)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all_bytes() {
        let data: Vec<u8> = (0u8..=255).collect();
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn empty() {
        assert_eq!(encode(&[]), "");
        assert_eq!(decode("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn known_value() {
        // 0b11_10_01_00 = 0b11100100 = 228
        // sounds: meow purr mew nya
        let encoded = encode(&[0b11100100]);
        let first_line: Vec<&str> = encoded.split_whitespace().take(4).collect();
        assert_eq!(first_line, vec!["meow", "purr", "mew", "nya"]);
        assert_eq!(decode(&encoded).unwrap(), vec![0b11100100]);
    }
}
