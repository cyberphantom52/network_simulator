mod connection;
mod interface;
mod physical;

use connection::Connection;
use physical::PhysicalLayer;

pub(self) type ConnectionMap = std::collections::HashMap<String, usize>;

pub enum ConnectionEndpoint<'a> {
    Device(Box<&'a mut dyn PhysicalLayer>),
    Connection(Connection),
}
