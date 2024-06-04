use crate::utils::calculate_crc;

pub trait ErrorControl {
    fn fcs(frame: &Vec<u8>) -> u32 {
        calculate_crc (frame)
    }
}
