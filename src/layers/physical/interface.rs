use crate::layers::datalink::MacAddr;
use super::link::Link;

/// Represents an interface that can be connected to another interface or a connection.
///
/// This is effectively an abstraction of a network interface card (NIC).
pub(crate) struct Interface {
    mac_addr: Option<MacAddr>,
    connection: Option<Link>,
}

impl Default for Interface {
    fn default() -> Self {
        Interface {
            mac_addr: Some(MacAddr::default()),
            connection: None,
        }
    }
}

impl Interface {
    /// Connects the interface to another interface or a connection.
    pub fn connect(&mut self, other: &mut Interface) {
        assert!(!self.is_connected());
        assert!(!other.is_connected());

        let (end, end2) = Link::connection();
        self.connection = Some(end);
        other.connection = Some(end2);
    }

    /// Disconnects the interface by dropping it's end of the connection.
    ///
    /// The actual connection will still be alive until all other interfaces
    /// connected to it are dropped.
    pub fn disconnect(&mut self) {
        assert!(self.is_connected());

        drop(self.connection.take());
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn send(&self, byte: u8) {
        assert!(self.is_connected());

        self.connection.as_ref().unwrap().send(byte).ok();
    }

    pub fn recv(&self) -> Option<u8> {
        assert!(self.is_connected());

        self.connection.as_ref().unwrap().recv().ok()
    }

    pub fn mac_addr(&self) -> Option<&MacAddr> {
        self.mac_addr.as_ref()
    }
}
