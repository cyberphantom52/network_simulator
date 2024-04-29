mod header;
mod datalink;
mod access_control;
mod error_control;
mod flow_control;
mod logical_link_control;

use access_control::AccessControl;
use error_control::ErrorControl;
use flow_control::FlowControl;
use logical_link_control::LogicalLinkControl;
pub use datalink::DataLinkLayer;

#[derive(Debug, Clone)]
pub struct MacAddr([u8; 6]);

impl Default for MacAddr {
    fn default() -> Self {
        MacAddr(rand::random())
    }
}

impl From<[u8; 6]> for MacAddr {
    fn from(bytes: [u8; 6]) -> Self {
        MacAddr(bytes)
    }
}

impl std::fmt::Display for MacAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
