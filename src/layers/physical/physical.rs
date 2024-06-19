use super::Link;
use crate::layers::NIC;
use std::sync::Arc;

pub trait PhysicalLayer {
    fn nic(&self) -> &NIC;

    fn connect(&self, other: Arc<impl PhysicalLayer>) {
        let (one, two) = Link::connection();
        self.nic().set_connection(Some(one));
        other.nic().set_connection(Some(two));
    }

    async fn disconnect(&self) {
        self.nic().set_connection(None);
    }

    async fn transmit(&self, byte: u8) {
        self.nic().transmit(byte).await;
    }

    async fn receive(&self) -> Option<u8> {
        self.nic().recieve().await
    }

    fn carrier_sense(&self) -> bool {
        self.nic().is_receiving()
    }

    fn transmitting(&self) -> bool {
        self.nic().transmitting()
    }

    fn collision_detect(&self) -> bool {
        self.carrier_sense() && self.transmitting()
    }
}
