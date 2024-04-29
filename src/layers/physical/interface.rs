use super::connection::Connection;

pub(crate) enum Endpoint<'a> {
    Interface(&'a mut Interface),
    Connection(Connection),
}

/// Represents an interface that can be connected to another interface or a connection.
///
/// This is effectively an abstraction of a network interface card (NIC).
pub(crate) struct Interface {
    connection: Option<Connection>,
}

impl Default for Interface {
    fn default() -> Self {
        Interface {
            connection: None,
        }
    }
}

impl Interface {
    /// Connects the interface to another interface or a connection.
    pub fn connect(&mut self, other: Endpoint) {
        assert!(!self.is_connected());

        match other {
            Endpoint::Interface(interface) => {
                assert!(!interface.is_connected());

                let connection = Connection::default();
                self.connection = Some(connection.clone());
                interface.connection = Some(connection);
            }
            Endpoint::Connection(connection) => {
                self.connection = Some(connection);
            }
        }
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
}
