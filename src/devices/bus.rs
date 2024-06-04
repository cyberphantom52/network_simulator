use super::hub::Hub;
use crate::layers::{Link, PhysicalLayer, NIC};
use crate::utils::Simulateable;
use futures::future::join_all;
use std::sync::Arc;

const N_JUNC: usize = 5;

pub struct Bus {
    junctions: [Hub; N_JUNC],
}

impl Default for Bus {
    fn default() -> Self {
        Bus {
            junctions: Default::default(),
        }
    }
}

impl Bus {
    pub fn index(number: usize) -> (usize, usize) {
        (number >> 16, number & 0xFFFF)
    }

    fn available_interface(&self) -> Option<usize> {
        for (i, junction) in self.junctions.iter().enumerate() {
            if let Some(interface) = junction.available_interface() {
                return Some(i << 16 | interface);
            }
        }

        None
    }
}

impl PhysicalLayer for Bus {
    fn nic(&self) -> &NIC {
        if let Some(interface) = self.available_interface() {
            let (junction, iface) = Bus::index(interface);
            &self.junctions[junction].interface(iface)
        } else {
            panic!("No NIC available")
        }
    }

    async fn connect(&self, other: Arc<impl PhysicalLayer>) {
        let (one, two) = Link::connection();
        if let Some(iface) = self.available_interface() {
            let (junction, iface) = Bus::index(iface);
            let iface = &self.junctions[junction].interface(iface);
            iface.set_connection(Some(one));
            other.nic().set_connection(Some(two));
        }
    }

    async fn disconnect(&self) {
        println!("Disconnect is not implemented for Bus");
    }
}

impl Simulateable for Bus {
    async fn tick(&self) {
        join_all(self.junctions.iter().map(|j| j.tick())).await;
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
        let bus = Arc::new(Bus::default());
        let dev1 = Arc::new(TestDevice::default());
        let dev2 = Arc::new(TestDevice::default());

        dev1.connect(bus.clone()).await;
        bus.connect(dev2.clone()).await;

        dev1.transmit(0x09).await;
        bus.tick().await;
        assert_eq!(dev2.receive().await, Some(0x09));
    }
}
