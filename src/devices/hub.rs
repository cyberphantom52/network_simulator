use std::sync::Arc;
use crate::layers::{Link, PhysicalLayer, NIC};
use crate::utils::Simulateable;
use tokio::sync::{Mutex, MutexGuard};

pub struct Hub {
    interfaces: [Mutex<NIC>; 8],
}

impl PhysicalLayer for Hub {
    async fn connect(&self, other: Arc<impl PhysicalLayer>) {
        let (one, two) = Link::connection();
        if let Some(mut iface) = self.available_interface().await {
            iface.set_connection(Some(one));
            other.nic().await.set_connection(Some(two));
        }
    }

    async fn nic(&self) -> MutexGuard<NIC> {
        // todo, this might be a problem since available_interface may return None
        self.available_interface().await.unwrap()
    }

    async fn disconnect(&mut self) {
        unimplemented!("Hub does not have a NIC")
    }
}

impl Hub {
    pub async fn available_interface(&self) -> Option<MutexGuard<NIC>> {
        for iface in self.interfaces.iter() {
            let unlocked = iface.lock().await;
            if !unlocked.is_connected() {
                return Some(unlocked);
            }
        }

        None
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
            let iface = iface.lock().await;
            if iface.is_connected() {
                connected_ifaces.push(iface);
            }
        }

        let bytes = connected_ifaces
            .iter_mut()
            .map(|iface| iface.recieve())
            .filter_map(|byte| byte)
            .collect::<Vec<_>>();

        connected_ifaces.iter_mut().for_each(|iface| {
            bytes.iter().for_each(|byte| iface.transmit(*byte));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDevice {
        nic: Mutex<NIC>,
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
