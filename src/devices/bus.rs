use super::hub::Hub;
use crate::layers::{PhysicalLayer, NIC};
use crate::utils::Simulateable;
use futures::future::join_all;
use std::sync::Arc;

const N_JUNC: usize = 5;

pub struct Bus {
    junctions: [Arc<Hub>; N_JUNC],
}

impl Default for Bus {
    fn default() -> Self {
        let junctions: [Arc<Hub>; N_JUNC] = Default::default();
        for i in 1..N_JUNC {
            junctions[i].connect(junctions[i - 1].clone());
        }

        Bus { junctions }
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

    #[tokio::test(flavor = "multi_thread")]
    async fn test_bus() {
        let bus = Arc::new(Bus::default());
        let devices: [Arc<TestDevice>; 32] = Default::default();

        for device in &devices {
            bus.connect(device.clone());
        }

        devices[0].transmit(0x09).await;
        for _ in 0..1 {
            bus.tick().await;
        }

        assert_eq!(devices[31].receive().await, Some(0x09));
    }
}
