use crate::layers::Identifier;
use super::interface::{Endpoint, Interface};
use super::{ConnectionEndpoint, ConnectionMap};

pub(crate) trait PhysicalLayer {
    /// Get the ID of the device
    fn id(&self) -> &Identifier;

    /// Send a frame
    ///
    /// If `None` is passed as the inteface number, the frame is broadcasted to all connected interfaces
    fn tansmit(&self, frame: Vec<u8>, interface: Option<usize>) {
        for byte in frame {
            match interface {
                Some(interface) => self.interface(interface).send(byte),
                None => {
                    for interface in self.interfaces() {
                        if interface.is_connected() {
                            interface.send(byte);
                        }
                    }
                }
            }
        }
    }

    /// Receive a frame from a random connected interface
    fn receive(&self) -> Option<u8> {
        use rand::seq::IteratorRandom;
        self.interfaces()
            .iter()
            .filter(|inteface| inteface.is_connected())
            .choose(&mut rand::thread_rng())
            .and_then(|inteface| inteface.recv())
    }

    /// Get a mappping of physical connections
    fn conn_map(&self) -> &ConnectionMap;

    /// Get a mutable mappping of physical connections
    fn conn_map_mut(&mut self) -> &mut ConnectionMap;

    /// Get shared reference to all interfaces
    ///
    /// The index of a interface in this list is implicitly the interface number
    fn interfaces(&self) -> &[Interface];

    /// Get mutable reference to all interfaces
    ///
    /// The index of a interface in this list is implicitly the interface number
    fn interfaces_mut(&mut self) -> &mut [Interface];

    /// Get an interface by it's number
    fn interface(&self, number: usize) -> &Interface {
        &self.interfaces()[number]
    }

    /// Get a mutable reference to an inteface from it's number
    fn interface_mut(&mut self, number: usize) -> &mut Interface {
        &mut self.interfaces_mut()[number]
    }

    /// Get the index of a free interface
    fn availabe_interface(&self) -> Option<usize> {
        self.interfaces()
            .iter()
            .position(|interface| !interface.is_connected())
    }

    /// Get interface number for a connection
    fn get_interface_for_connection(&self, device_id: &Identifier) -> Option<usize> {
        self.conn_map().get::<String>(&device_id.to_string()).copied()
    }

    /// Connect to a device
    fn connect(&mut self, other: ConnectionEndpoint) {
        match other {
            ConnectionEndpoint::Device(device) => {
                let interface = self.availabe_interface();
                let other_interface = device.availabe_interface();

                match (interface, other_interface) {
                    (Some(interface), Some(other_interface)) => {
                        self.interface_mut(interface).connect(Endpoint::Interface(device.interface_mut(other_interface)));
                        self.conn_map_mut().insert(device.id().to_string(), interface);
                        device.conn_map_mut().insert(self.id().to_string(), other_interface);
                    }
                    _ => {
                        panic!("No free interface available");
                    }
                }
            }
            ConnectionEndpoint::Connection(connection) => {
                let interface_id = self.availabe_interface();

                match interface_id {
                    Some(interface_id) => {
                        let conn_id = connection.id().to_string();
                        self.interface_mut(interface_id)
                            .connect(Endpoint::Connection(connection));
                        self.conn_map_mut()
                            .insert(conn_id, interface_id);
                    }
                    None => {
                        panic!("No free interface available");
                    }
                }
            }
        }
    }

    /// Disconnect from a device
    fn disconnect(&mut self, other: ConnectionEndpoint) {
        match other {
            ConnectionEndpoint::Device(device) => {
                let interface_id = self.get_interface_for_connection(device.id());
                let other_interface_id = device.get_interface_for_connection(self.id());

                match (interface_id, other_interface_id) {
                    (Some(interface_id), Some(other_interface_id)) => {
                        let interface = self.interface_mut(interface_id);
                        let other_interface = device.interface_mut(other_interface_id);
                        interface.disconnect();
                        other_interface.disconnect();

                        self.conn_map_mut().remove::<String>(&device.id().to_string());
                        device.conn_map_mut().remove::<String>(&self.id().to_string());
                    }
                    _ => {
                        panic!("Connection not found");
                    }
                }
            }
            ConnectionEndpoint::Connection(connection) => {
                let interface_id = self.conn_map()
                    .get::<String>(&connection.id().into())
                    .copied();
                match interface_id {
                    Some(interface_id) => {
                        let interface = self.interface_mut(interface_id);
                        interface.disconnect();
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
