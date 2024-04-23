use crate::layers::datalink::Frame;
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    RwLock,
};

pub type PortId = usize;

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
    connection: Option<Connection>,
}

impl Default for PhysicalPort {
    fn default() -> Self {
        PhysicalPort { connection: None }
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
