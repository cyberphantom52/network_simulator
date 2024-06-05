use super::{physical::Link, MacAddr};
use futures::{Future, FutureExt};
use tokio::sync::{
    mpsc::error::{TryRecvError, TrySendError},
    RwLock,
};

/// Abstraction of a network interface card (NIC).
///
/// Provides physical layer primitives for sending and receiving data.
/// As well as Layer 2 primitives for addressing and switching.
pub struct NIC {
    mac: MacAddr,
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
        self.mac.clone()
    }

    pub fn transmitting(&self) -> impl Future<Output = bool> + '_ {
        self.transmitting.read().map(|guard| *guard)
    }

    pub fn set_transmitting(&self, transmitting: bool) -> impl Future<Output = ()> + '_ {
        self.transmitting
            .write()
            .map(move |mut guard| *guard = transmitting)
    }

    pub fn set_connection(&self, connection: Option<Link>) -> impl Future<Output = ()> + '_ {
        self.connection
            .write()
            .map(move |mut guard| *guard = connection)
    }

    pub fn is_receiving(&self) -> impl Future<Output = bool> + '_ {
        self.connection
            .read()
            .map(|lock| lock.as_ref().map_or(false, |conn| conn.is_recieving()))
    }

    pub fn is_connected(&self) -> impl Future<Output = bool> + '_ {
        self.connection.read().map(|lock| lock.is_some())
    }

    pub async fn transmit(&self, byte: u8) {
        if let Some(conn) = self.connection.read().await.as_ref() {
            let status = conn.send(byte);
            match status {
                Ok(()) => (),
                Err(e) => match e {
                    TrySendError::Closed(_) => {
                        self.set_connection(None).await;
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
                            self.set_connection(None).await;
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
        nic1.set_connection(Some(one)).await;
        nic2.set_connection(Some(two)).await;
        assert!(nic1.is_connected().await);
        assert!(nic2.is_connected().await);
        nic1.set_connection(None).await;

        nic2.recieve().await;
        assert!(!nic1.is_connected().await);
        assert!(!nic2.is_connected().await);
    }

    #[tokio::test]
    async fn test_transmit_recieve() {
        let nic1 = NIC::default();
        let nic2 = NIC::default();

        let (one, two) = Link::connection();
        nic1.set_connection(Some(one)).await;
        nic2.set_connection(Some(two)).await;

        nic1.transmit(0x42).await;
        assert_eq!(nic2.recieve().await, Some(0x42));
    }
}
