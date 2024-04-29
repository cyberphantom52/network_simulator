mod connection;
mod interface;
mod physical;

pub use physical::PhysicalLayer;
use connection::Connection;

pub(self) type ConnectionMap = std::collections::HashMap<String, usize>;

pub enum ConnectionEndpoint<'a> {
    Device(Box<&'a mut dyn PhysicalLayer>),
    Connection(Connection),
}
