mod error_control;
mod flow_control;
mod header;
mod logical_link_control;
mod media_access_control;

pub use error_control::ErrorControl;
pub use flow_control::FlowControl;
pub use logical_link_control::LogicalLinkControl;
pub use media_access_control::{AccessControl, TransmitState, ReceiveState};

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
