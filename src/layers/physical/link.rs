use std::sync::Mutex;
use tokio::sync::mpsc::{channel, error::{SendError, TryRecvError}, Receiver, Sender};

/// Represents a one way link between two endpoints.
///
/// A connection is established by creating a pair of links with interchangable senders and receivers.
pub(super) struct Link {
    tx: Sender<u8>,
    rx: Mutex<Receiver<u8>>,
}

impl Link {
    fn oneway(tx: Sender<u8>, rx: Receiver<u8>) -> Self {
        Self {tx, rx: Mutex::new(rx),
        }
    }

    /// Create a new connection and return it as a pair of one way links.
    pub fn connection() -> (Self, Self) {
        let (tx1, rx1) = channel(2000);
        let (tx2, rx2) = channel(2000);
        (Self::oneway(tx1, rx2), Self::oneway(tx2, rx1))
    }

    /// Send a byte of data through the link.
    ///
    /// The reciever of the data needs to call `recv` on it's end of the link.
    pub async fn send(&self, data: u8) -> Result<(), SendError<u8>> {
        self.tx.send(data).await
    }

    /// Receive a byte of data from the link.
    pub fn recv(&self) -> Result<u8, TryRecvError> {
        self.rx.lock().unwrap().try_recv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_link() {
        let (a, b) = Link::connection();
        a.send(42).await.ok();
        assert_eq!(a.recv().is_err(), true);
        assert_eq!(b.recv().unwrap(), 42);
    }
}
