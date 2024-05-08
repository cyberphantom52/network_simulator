use super::interface::Interface;
use super::ConnectionMap;
use crate::layers::Identifier;
use std::sync::{Arc, Mutex};

pub trait PhysicalLayer {
    /// Get the ID of the device
    fn id(&self) -> &Identifier;

    /// Send a byte through the channel
    ///
    /// If `None` is passed as the inteface number, the byte is broadcasted to all connected interfaces
    async fn transmit(&self, byte: u8, interface: Option<usize>) {
        match interface {
            Some(interface) => self.interface(interface).send(byte).await,
            None => {
                for interface in self.interfaces() {
                    if interface.is_connected() {
                        interface.send(byte).await;
                    }
                }
            }
        }
    }

    /// Receive a byte from the specified connected interface
    ///
    /// If `None` is passed as the inteface number, a random connected interface is selected
    async fn receive(&self, interface: Option<usize>) -> Option<(u8, usize)> {
        match interface {
            Some(interface) => self
                .interface(interface)
                .recv()
                .map(|byte| (byte, interface)),
            None => {
                use rand::seq::IteratorRandom;
                self.interfaces()
                    .iter()
                    .enumerate()
                    .filter(|(_, inteface)| inteface.is_connected())
                    .choose(&mut rand::thread_rng())
                    .and_then(|(index, inteface)| inteface.recv().map(|byte| (byte, index)))
            }
        }
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
    async fn connect(&mut self, other: Arc<Mutex<impl PhysicalLayer>>) {
        let mut other = other.lock().unwrap();
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
    async fn disconnect(&mut self, other: Arc<Mutex<impl PhysicalLayer>>) {
        let mut other = other.lock().unwrap();
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

#[cfg(test)]
mod tests {
    use crate::arc_mutex;

    use super::*;

    struct TestPhysicalLayer {
        id: Identifier,
        interfaces: [Interface; 2],
        conn_map: ConnectionMap,
    }

    impl TestPhysicalLayer {
        fn new(id: &str) -> Self {
            Self {
                id: Identifier::Name(id.to_string()),
                interfaces: Default::default(),
                conn_map: Default::default(),
            }
        }
    }

    impl PhysicalLayer for TestPhysicalLayer {
        fn id(&self) -> &Identifier {
            &self.id
        }

        fn conn_map(&self) -> &ConnectionMap {
            &self.conn_map
        }

        fn conn_map_mut(&mut self) -> &mut ConnectionMap {
            &mut self.conn_map
        }

        fn interfaces(&self) -> &[Interface] {
            &self.interfaces
        }

        fn interfaces_mut(&mut self) -> &mut [Interface] {
            &mut self.interfaces
        }
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut pl1 = TestPhysicalLayer::new("test1");
        let pl2 = arc_mutex!(TestPhysicalLayer::new("test2"));

        pl1.connect(pl2.clone()).await;
        assert_eq!(pl1.get_interface_for_connection(&pl2.lock().unwrap().id()), Some(0));
        assert_eq!(pl2.lock().unwrap().get_interface_for_connection(&pl1.id()), Some(0));

        pl1.disconnect(pl2.clone()).await;
        assert_eq!(pl1.get_interface_for_connection(&pl2.lock().unwrap().id()),None);
        assert_eq!(pl2.lock().unwrap().get_interface_for_connection(&pl1.id()),None);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_connect_no_free_interface() {
        let mut physical_layer1 = TestPhysicalLayer::new("test1");
        let physical_layer2 = Arc::new(Mutex::new(TestPhysicalLayer::new("test2")));

        physical_layer1.connect(physical_layer2.clone()).await;
        physical_layer1.connect(physical_layer2.clone()).await;
        physical_layer1.connect(physical_layer2.clone()).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn test_disconnect_no_connection() {
        let mut physical_layer1 = TestPhysicalLayer::new("test1");
        let physical_layer2 = Arc::new(Mutex::new(TestPhysicalLayer::new("test2")));

        physical_layer1.disconnect(physical_layer2.clone()).await;
    }

    #[tokio::test]
    async fn test_transmit() {
        let mut physical_layer1 = TestPhysicalLayer::new("test1");
        let physical_layer2 = Arc::new(Mutex::new(TestPhysicalLayer::new("test2")));

        physical_layer1.connect(physical_layer2.clone()).await;

        physical_layer1.transmit(0x01, None).await;
        physical_layer2.lock().unwrap().transmit(0x02, None).await;
        assert_eq!(physical_layer2.lock().unwrap().receive(Some(0)).await, Some((0x01, 0)));
        assert_eq!(physical_layer1.receive(Some(0)).await, Some((0x02, 0)));
    }

    #[tokio::test]
    async fn test_receive_no_data() {
        let mut physical_layer1 = TestPhysicalLayer::new("test1");
        let physical_layer2 = Arc::new(Mutex::new(TestPhysicalLayer::new("test2")));

        physical_layer1.connect(physical_layer2.clone()).await;
        assert_eq!(physical_layer1.receive(Some(0)).await, None);
    }

    #[tokio::test]
    async fn test_transmit_multiple_connections() {
        let mut physical_layer1 = TestPhysicalLayer::new("test1");
        let physical_layer2 = Arc::new(Mutex::new(TestPhysicalLayer::new("test2")));
        let physical_layer3 = Arc::new(Mutex::new(TestPhysicalLayer::new("test3")));

        physical_layer1.connect(physical_layer2.clone()).await;
        physical_layer1.connect(physical_layer3.clone()).await;
        physical_layer2.lock().unwrap().transmit(0x01, Some(0)).await;
        physical_layer3.lock().unwrap().transmit(0x02, Some(0)).await;

        assert_eq!(physical_layer1.receive(Some(0)).await, Some((0x01, 0)));
        assert_eq!(physical_layer1.receive(Some(1)).await, Some((0x02, 1)));
    }
}
