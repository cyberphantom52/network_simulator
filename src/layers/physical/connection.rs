use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::{
    broadcast::{self, error::{SendError, RecvError, TryRecvError}, Receiver, Sender},
    RwLock,
};

/// Represents a connection between two or more interfaces.
///
/// The connection can be point-to-point or broadcast depending on the number of
/// interfaces connected to it.
pub(crate) struct Connection {
    id: String,
    sender: Sender<u8>,
    receiver: RwLock<Receiver<u8>>,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            sender: self.sender.clone(),
            receiver: RwLock::new(self.sender.subscribe()),
        }
    }
}

impl Default for Connection {
    fn default() -> Self {
        let (sender, receiver) = broadcast::channel(3000);
        let id = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>();
        Self {
            id: format!("Connection-{}", id),
            sender,
            receiver: RwLock::new(receiver),
        }
    }
}

impl Connection {
    /// Returns the unique identifier of the connection.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Send a byte of data through the connection.
    ///
    /// The recievers of the data need to call `recv` to get the data.
    pub fn send(&self, data: u8) -> Result<usize, SendError<u8>> {
        self.sender.send(data)
    }

    /// Receive a byte of data from the connection.
    pub fn recv(&self) -> Result<u8, TryRecvError> {
        // TODO: Handle the actual error variants.
        self.receiver.blocking_write().try_recv()
    }
}
