use super::ConnectionMap;
use super::port::{PhysicalPort, PortNumber};
use crate::layers::{ConnectionTarget, Identifier};

pub(crate) trait PhysicalLayer {
    /// Get the ID of the device
    fn id(&self) -> &Identifier;

    /// Send a frame
    ///
    /// If `None` is passed as the port, the frame is broadcasted to all connected ports
    fn tansmit(&self, frame: Vec<u8>, port: Option<PortNumber>) {
        for byte in frame {
            match port {
                Some(port) => self.port(port).send(byte),
                None => {
                    for port in self.ports() {
                        if port.is_connected() {
                            port.send(byte);
                        }
                    }
                }
            }
        }
    }

    /// Receive a frame from a random connected port
    fn receive(&self) -> Option<u8> {
        use rand::seq::IteratorRandom;
        self.ports()
            .iter()
            .filter(|port| port.is_connected())
            .choose(&mut rand::thread_rng())
            .and_then(|port| port.recv())
    }

    /// Get a mappping of physical connections
    fn conn_map(&self) -> &ConnectionMap;

    /// Get a mutable mappping of physical connections
    fn conn_map_mut(&mut self) -> &mut ConnectionMap;

    /// Get all physical ports
    ///
    /// The index of a port in this list is implicitly the port number
    fn ports(&self) -> &[PhysicalPort];

    /// Get mutable reference to all physical ports
    ///
    /// The index of a port in this list is implicitly the port number
    fn ports_mut(&mut self) -> &mut [PhysicalPort];

    /// Get a physical port by port number
    fn port(&self, port: PortNumber) -> &PhysicalPort {
        &self.ports()[port]
    }

    /// Get a mutable reference to a physical port by port number
    fn port_mut(&mut self, port: PortNumber) -> &mut PhysicalPort {
        &mut self.ports_mut()[port]
    }

    /// Get port id of a free port
    fn get_free_port(&self) -> Option<PortNumber> {
        self.ports()
            .iter()
            .position(|port| !port.is_connected())
            .map(|port| PortNumber::from(port))
    }

    /// Get port id for a connection
    fn get_port_for_connection(&self, other: &Identifier) -> Option<PortId> {
        self.conn_map().get(&other.to_string()).copied()
    }

    /// Connect to a device
    fn connect(&mut self, other: ConnectionTarget) {
        match other {
            ConnectionTarget::Device(device) => {
                let port = self.get_free_port();
                let other_port = device.get_free_port();

                match (port, other_port) {
                    (Some(port), Some(other_port)) => {
                        self.port_mut(port).connect(device.port_mut(other_port));
                        self.conn_map_mut().insert(device.id().to_string(), port);
                        device.conn_map_mut().insert(self.id().to_string(), other_port);
                    }
                    _ => {
                        panic!("No free ports available");
                    }
                }
            }
            ConnectionTarget::Connection(connection) => {
                let port_id = self.get_free_port();

                match port_id {
                    Some(port_id) => {
                        let port = self.port_mut(port_id);
                        port.set_connection(connection.clone());
                        self.conn_map_mut().insert(connection.id().into(), port_id);
                    }
                    None => {
                        panic!("No free port available");
                    }
                }
            }
        }
    }

    /// Disconnect from a device
    fn disconnect(&mut self, other: ConnectionTarget) {
        match other {
            ConnectionTarget::Device(device) => {
                let port_id = self.get_port_for_connection(&device.id());
                let other_id = device.get_port_for_connection(&self.id());

                match (port_id, other_id) {
                    (Some(port_id), Some(other_id)) => {
                        let port = self.port_mut(port_id);
                        let other_port = device.port_mut(other_id);
                        port.disconnect();
                        other_port.disconnect();

                        self.conn_map_mut().remove::<String>(&device.id().to_string());
                        device.conn_map_mut().remove::<String>(&self.id().to_string());
                    }
                    _ => {
                        panic!("Connection not found");
                    }
                }
            }
            ConnectionTarget::Connection(connection) => {
                let port_id = self.conn_map()
                    .get::<String>(&connection.id().into())
                    .copied();
                match port_id {
                    Some(port_id) => {
                        let port = self.port_mut(port_id);
                        port.disconnect();
                        self.conn_map_mut()
                            .remove::<String>(&connection.id().into());
                    }
                    None => {
                        panic!("Connection not found");
                    }
                }
            }
        }
    }
}
