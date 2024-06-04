/*
  Reference:
    http://www.sunshine2k.de/articles/coding/crc/understanding_crc.html
    https://reveng.sourceforge.io/crc-catalogue/all.htm
*/
const CRC32_MEF: CrcModel = CrcModel {
    initial: 0xFFFF_FFFF,
    final_xor: 0x0000_0000,
    polynomial: 0x741b8cd7,
    reflect_input: true,
    reflect_output: true,
};

struct CrcModel {
    initial: u32,
    polynomial: u32,
    final_xor: u32,
    reflect_input: bool,
    reflect_output: bool,
}

pub fn calculate_crc(bytes: &[u8]) -> u32 {
    let mut crc = CRC32_MEF.initial;
    for &byte in bytes {
        let data = match CRC32_MEF.reflect_input {
            true => byte.reverse_bits(),
            false => byte,
        };

        crc ^= (data as u32) << 24;
        for _ in 0..8 {
            if crc & 0x8000_0000 != 0 {
                crc = (crc << 1) ^ CRC32_MEF.polynomial;
            } else {
                crc <<= 1;
            }
        }
    }
    crc = match CRC32_MEF.reflect_output {
        true => crc.reverse_bits(),
        false => crc,
    };
    crc ^ CRC32_MEF.final_xor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc() {
        let data = vec![0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39];
        let crc = calculate_crc(&data);
        assert_eq!(crc, 0xd2c22f51);
    }

    #[test]
    fn validate_crc() {
        let mut data = vec![0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39];
        let crc = calculate_crc(&data);
        data.extend_from_slice(&crc.to_le_bytes());
        assert_eq!(calculate_crc(&data), 0);
    }
}
