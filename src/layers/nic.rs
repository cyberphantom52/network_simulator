use super::{physical::Link, MacAddr};
use futures::executor::block_on;
use tokio::sync::{
    mpsc::error::{TryRecvError, TrySendError},
    RwLock,
};

/// Abstraction of a network interface card (NIC).
///
/// Provides physical layer primitives for sending and receiving data.
/// As well as Layer 2 primitives for addressing and switching.
pub struct NIC {
    mac: RwLock<MacAddr>,
    transmitting: RwLock<bool>,
    connection: RwLock<Option<Link>>,
}

impl Default for NIC {
    fn default() -> Self {
        NIC {
            mac: Default::default(),
            transmitting: RwLock::new(false),
            connection: RwLock::new(None),
        }
    }
}

impl NIC {
    pub fn mac(&self) -> MacAddr {
        block_on(async { self.mac.read().await.clone() })
    }

    pub fn transmitting(&self) -> bool {
        block_on(async { self.transmitting.read().await.clone() })
    }

    pub fn set_transmitting(&self, transmitting: bool) {
        block_on(async { *self.transmitting.write().await = transmitting })
    }

    pub fn set_connection(&self, connection: Option<Link>) {
        block_on(async {
            *self.connection.write().await = connection;
        })
    }

    pub fn is_receiving(&self) -> bool {
        block_on(async {
            if let Some(conn) = self.connection.read().await.as_ref() {
                conn.is_recieving()
            } else {
                false
            }
        })
    }

    pub fn is_connected(&self) -> bool {
        block_on(async { self.connection.read().await.is_some() })
    }

    pub async fn transmit(&self, byte: u8) {
        if let Some(conn) = self.connection.read().await.as_ref() {
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

    pub async fn recieve(&self) -> Option<u8> {
        let mut handle = self.connection.write().await;
        if let Some(conn) = handle.as_mut() {
            return match conn.recv() {
                Ok(byte) => Some(byte),
                Err(e) => {
                    match e {
                        TryRecvError::Disconnected => {
                            drop(handle);
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
        let nic1 = NIC::default();
        let nic2 = NIC::default();

        let (one, two) = Link::connection();
        nic1.set_connection(Some(one));
        nic2.set_connection(Some(two));
        assert!(nic1.is_connected());
        assert!(nic2.is_connected());
        nic1.set_connection(None);

        nic2.recieve().await;
        assert!(!nic1.is_connected());
        assert!(!nic2.is_connected());
    }

    #[tokio::test]
    async fn test_transmit_recieve() {
        let nic1 = NIC::default();
        let nic2 = NIC::default();

        let (one, two) = Link::connection();
        nic1.set_connection(Some(one));
        nic2.set_connection(Some(two));

        nic1.transmit(0x42).await;
        assert_eq!(nic2.recieve().await, Some(0x42));
    }
}
