use crate::layers::Identifier;
use super::interface::Interface;
use super::ConnectionMap;

pub trait PhysicalLayer {
    /// Get the ID of the device
    fn id(&self) -> &Identifier;

    /// Send a byte through the channel
    ///
    /// If `None` is passed as the inteface number, the byte is broadcasted to all connected interfaces
    fn tansmit(&self, byte: u8, interface: Option<usize>) {
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

    /// Receive a byte from a random connected interface
    ///
    /// Also returns the interface number from which the byte was received
    fn receive(&self) -> Option<(u8, usize)> {
        use rand::seq::IteratorRandom;
        self.interfaces()
            .iter()
            .enumerate()
            .filter(|(_, inteface)| inteface.is_connected())
            .choose(&mut rand::thread_rng())
            .and_then(|(index, inteface)| inteface.recv().map(|byte| (byte, index)))
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
    fn connect(&mut self, other: Box<&mut dyn PhysicalLayer>) {
        let interface = self.availabe_interface();
        let other_interface = other.availabe_interface();

        match (interface, other_interface) {
            (Some(interface), Some(other_interface)) => {
                self.interface_mut(interface).connect(other.interface_mut(other_interface));
                self.conn_map_mut().insert(other.id().to_string(), interface);
                other.conn_map_mut().insert(self.id().to_string(), other_interface);
            }
            _ => {
                panic!("No free interface available");
            }
        }
    }

    /// Disconnect from a device
    fn disconnect(&mut self, other: Box<&mut dyn PhysicalLayer>) {
        let interface_id = self.get_interface_for_connection(other.id());
        let other_interface_id = other.get_interface_for_connection(self.id());

        match (interface_id, other_interface_id) {
            (Some(interface_id), Some(other_interface_id)) => {
                let interface = self.interface_mut(interface_id);
                let other_interface = other.interface_mut(other_interface_id);
                interface.disconnect();
                other_interface.disconnect();

                self.conn_map_mut().remove::<String>(&other.id().to_string());
                other.conn_map_mut().remove::<String>(&self.id().to_string());
            }
            _ => {
                panic!("Connection not found");
            }
        }
    }
}
