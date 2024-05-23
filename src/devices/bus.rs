use std::sync::Arc;
use super::hub::Hub;
use crate::layers::{Link, PhysicalLayer, NIC};
use crate::utils::Simulateable;
use futures::future::join_all;
use tokio::sync::MutexGuard;

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
    async fn available_interface(&self) -> Option<MutexGuard<NIC>> {
        for iface in self.junctions.iter() {
            if let Some(iface) = iface.available_interface().await {
                return Some(iface);
            }
        }

        None
    }
}

impl PhysicalLayer for Bus {
    async fn nic(&self) -> tokio::sync::MutexGuard<crate::layers::NIC> {
        // todo, this might be a problem since available_interface may return None
        self.available_interface().await.unwrap()
    }

    async fn connect(&self, other: Arc<impl PhysicalLayer>) {
        let (one, two) = Link::connection();
        if let Some(mut iface) = self.available_interface().await {
            iface.set_connection(Some(one));
            other.nic().await.set_connection(Some(two));
        }
    }

    async fn disconnect(&mut self) {
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
        nic: tokio::sync::Mutex<NIC>,
    }

    impl Default for TestDevice {
        fn default() -> Self {
            TestDevice {
                nic: Default::default(),
            }
        }
    }
    impl PhysicalLayer for TestDevice {
        async fn nic(&self) -> tokio::sync::MutexGuard<NIC> {
            self.nic.lock().await
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
