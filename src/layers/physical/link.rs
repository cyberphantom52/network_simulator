use tokio::sync::mpsc::{
    channel,
    error::{TryRecvError, TrySendError},
    Receiver, Sender,
};

/// A `Physical Layer` primitive that represents a one way link between two endpoints.
///
/// A connection is established by creating a pair of links with interchanged senders and receivers.
pub struct Link {
    tx: Sender<u8>,
    rx: Receiver<u8>,
}

impl Link {
    fn oneway(tx: Sender<u8>, rx: Receiver<u8>) -> Self {
        Self { tx, rx }
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
    pub fn send(&self, data: u8) -> Result<(), TrySendError<u8>> {
        self.tx.try_send(data)
    }

    /// Receive a byte of data from the link.
    pub fn recv(&mut self) -> Result<u8, TryRecvError> {
        self.rx.try_recv()
    }

    pub fn is_recieving(&self) -> bool {
        !self.rx.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_link() {
        let (mut a, mut b) = Link::connection();
        a.send(42).ok();
        assert_eq!(a.recv().is_err(), true);
        assert_eq!(b.recv().unwrap(), 42);
    }
}
