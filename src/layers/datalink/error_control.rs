pub trait ErrorControl {
    fn fcs(frame: Vec<u8>) -> u32;
}
