use crate::layers::datalink::Frame;
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    RwLock,
};

/// Wrapper around a port number
///
/// This is used to index into the list of physical ports
#[derive(Debug, Clone, Copy)]
pub struct PortNumber(u8);

impl Index<PortNumber> for [PhysicalPort] {
    type Output = PhysicalPort;

    fn index(&self, index: PortNumber) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl IndexMut<PortNumber> for [PhysicalPort] {
    fn index_mut(&mut self, index: PortNumber) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

impl From<usize> for PortNumber {
    fn from(index: usize) -> Self {
        PortNumber(index as u8)
    }
}

#[derive(Debug)]
pub struct Connection {
    id: String,
    pub outbound: Sender<Frame>,
    pub inbound: RwLock<Receiver<Frame>>,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Connection {
            id: self.id.clone(),
            outbound: self.outbound.clone(),
            inbound: RwLock::new(self.outbound.subscribe()),
        }
    }
}

impl Connection {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(1540);
        let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        Connection {
            id: format!("bus_{}", random),
            outbound: tx,
            inbound: RwLock::new(rx),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn send(&self, packet: Frame) {
        self.outbound.send(packet).unwrap();
    }

    pub fn recv(&self) -> Option<Frame> {
        self.inbound.blocking_write().try_recv().ok()
    }
}

#[derive(Debug)]
pub struct PhysicalPort {
    mac: Option<MacAddr>,
    connection: Option<Connection>,
}

impl Default for PhysicalPort {
    fn default() -> Self {
        PhysicalPort {
            mac: Some(MacAddr::new()),
            connection: None,
        }
    }
}

impl PhysicalPort {
    pub fn connect(&mut self, other: &mut PhysicalPort) {
        assert!(!self.is_connected());

        let connection = Connection::new();
        other.connection = Some(connection.clone());
        self.connection = Some(connection);
    }

    pub fn set_connection(&mut self, connection: Connection) {
        self.connection = Some(connection);
    }

    pub fn disconnect(&mut self) {
        assert!(self.is_connected());

        drop(self.connection.take());
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn send(&self, packet: Frame) {
        assert!(self.is_connected());

        self.connection.as_ref().unwrap().send(packet);
    }

    pub fn recv(&self) -> Option<Frame> {
        assert!(self.is_connected());

        self.connection.as_ref().unwrap().recv()
    }
}
