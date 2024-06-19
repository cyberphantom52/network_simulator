use super::{physical::Link, MacAddr};
use tokio::sync::mpsc::error::{TryRecvError, TrySendError};
/// Abstraction of a network interface card (NIC).
///
/// Provides physical layer primitives for sending and receiving data.
/// As well as Layer 2 primitives for addressing and switching.
pub struct NIC {
    transmitting: bool,
    connection: Option<Link>,
    mac_addr: MacAddr,
}

impl Default for NIC {
    fn default() -> Self {
        NIC {
            transmitting: false,
            connection: None,
            mac_addr: MacAddr::default(),
        }
    }
}

impl NIC {
    pub fn connection(&self) -> Option<&Link> {
        self.connection.as_ref()
    }

    pub fn transmitting(&self) -> bool {
        self.transmitting
    }

    pub fn set_transmitting(&mut self, transmitting: bool) {
        self.transmitting = transmitting;
    }

    pub fn set_connection(&mut self, connection: Option<Link>) {
        self.connection = connection;
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn transmit(&mut self, byte: u8) {
        if let Some(conn) = self.connection.as_ref() {
            let status = conn.send(byte);
            match status {
                Ok(()) => (),
                Err(e) => match e {
                    TrySendError::Closed(_) => {
                        self.set_connection(None);
                    }
                    _ => (),
                },
            }
        }
    }

    pub fn recieve(&mut self) -> Option<u8> {
        if let Some(conn) = self.connection.as_mut() {
            return match conn.recv() {
                Ok(byte) => Some(byte),
                Err(e) => {
                    match e {
                        TryRecvError::Disconnected => {
                            self.set_connection(None);
                        }
                        _ => (),
                    }
                    None
                }
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auto_disconnect() {
        let mut nic1 = NIC::default();
        let mut nic2 = NIC::default();

        let (one, two) = Link::connection();
        nic1.set_connection(Some(one));
        nic2.set_connection(Some(two));
        assert!(nic1.connection().is_some());
        assert!(nic2.connection().is_some());
        nic1.set_connection(None);

        nic2.recieve();
        assert!(nic1.connection().is_none());
        assert!(nic2.connection().is_none());
    }

    #[test]
    fn test_transmit_recieve() {
        let mut nic1 = NIC::default();
        let mut nic2 = NIC::default();

        let (one, two) = Link::connection();
        nic1.set_connection(Some(one));
        nic2.set_connection(Some(two));

        nic1.transmit(0x42);
        assert_eq!(nic2.recieve(), Some(0x42));
    }
}
