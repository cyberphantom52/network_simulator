use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MacAddr([u8; 6]);

impl MacAddr {
    pub fn new() -> Self {
        MacAddr(rand::random())
    }

    pub fn from(str: &str) -> Self {
        let mut bytes = [0; 6];
        let mut i = 0;
        for byte in str.split(':') {
            bytes[i] = u8::from_str_radix(byte, 16).unwrap();
            i += 1;
        }
        MacAddr(bytes)
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
