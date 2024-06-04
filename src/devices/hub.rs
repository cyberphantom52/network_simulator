use crate::layers::{Link, PhysicalLayer, NIC};
use crate::utils::Simulateable;
use std::sync::Arc;

pub struct Hub {
    interfaces: [NIC; 8],
}

impl PhysicalLayer for Hub {
    async fn connect(&self, other: Arc<impl PhysicalLayer>) {
        let (one, two) = Link::connection();
        if let Some(interface) = self.available_interface() {
            let iface = &self.interfaces[interface];
            iface.set_connection(Some(one));
            other.nic().set_connection(Some(two));
        }
    }

    fn nic(&self) -> &NIC {
        if let Some(interface) = self.available_interface() {
            &self.interfaces[interface]
        } else {
            panic!("No NIC available")
        }
    }

    async fn disconnect(&self) {
        unimplemented!("Hub does not have a NIC")
    }
}

impl Hub {
    pub fn available_interface(&self) -> Option<usize> {
        for (i, iface) in self.interfaces.iter().enumerate() {
            if !iface.is_connected() {
                return Some(i);
            }
        }

        None
    }

    pub fn interface(&self, index: usize) -> &NIC {
        &self.interfaces[index]
    }
}

impl Default for Hub {
    fn default() -> Self {
        Hub {
            interfaces: Default::default(),
        }
    }
}

impl Simulateable for Hub {
    async fn tick(&self) {
        let mut connected_ifaces = Vec::new();
        for iface in self.interfaces.iter() {
            if iface.is_connected() {
                connected_ifaces.push(iface);
            }
        }

        let mut bytes = Vec::new();
        for iface in connected_ifaces.iter() {
            if let Some(byte) = iface.recieve().await {
                bytes.push(byte);
            }
        }

        for iface in connected_ifaces.iter() {
            for byte in bytes.iter() {
                iface.transmit(*byte).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDevice {
        nic: NIC,
    }

    impl Default for TestDevice {
        fn default() -> Self {
            TestDevice {
                nic: Default::default(),
            }
        }
    }
    impl PhysicalLayer for TestDevice {
        fn nic(&self) -> &NIC {
            &self.nic
        }
    }

    #[tokio::test]
    async fn test_hub() {
        let hub = Arc::new(Hub::default());
        let dev1 = Arc::new(TestDevice::default());
        let dev2 = Arc::new(TestDevice::default());

        dev1.connect(hub.clone()).await;
        hub.connect(dev2.clone()).await;

        dev1.transmit(0x09).await;
        hub.tick().await;
        assert_eq!(dev2.receive().await, Some(0x09));
    }
}
