use crate::utils::CRC32_MEF;

pub trait ErrorControl {
    fn calculate_checksum(&self, bytes: &[u8]) -> u32 {
        let mut crc = CRC32_MEF.initial();
        for &byte in bytes {
            let data = match CRC32_MEF.reflect_input() {
                true => byte.reverse_bits(),
                false => byte,
            };

            crc ^= (data as u32) << 24;
            for _ in 0..8 {
                if crc & 0x8000_0000 != 0 {
                    crc = (crc << 1) ^ CRC32_MEF.polynomial();
                } else {
                    crc <<= 1;
                }
            }
        }
        crc = match CRC32_MEF.reflect_output() {
            true => crc.reverse_bits(),
            false => crc,
        };
        crc ^ CRC32_MEF.final_xor()
    }
}
