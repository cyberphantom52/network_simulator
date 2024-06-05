use futures::{future::join, Future, FutureExt};

use super::Link;
use crate::layers::NIC;
use std::sync::Arc;

pub trait PhysicalLayer {
    fn nic(&self) -> &NIC;

    async fn connect(&self, other: Arc<impl PhysicalLayer>) {
        let (one, two) = Link::connection();
        self.nic().set_connection(Some(one)).await;
        other.nic().set_connection(Some(two)).await;
    }

    async fn disconnect(&self) {
        self.nic().set_connection(None).await;
    }

    async fn transmit(&self, byte: u8) {
        self.nic().transmit(byte).await;
    }

    async fn receive(&self) -> Option<u8> {
        self.nic().recieve().await
    }

    fn carrier_sense(&self) -> impl Future<Output = bool> {
        self.nic().is_receiving()
    }

    fn transmitting(&self) -> impl Future<Output = bool> {
        self.nic().transmitting()
    }

    fn collision_detect(&self) -> impl Future<Output = bool> + '_ {
        join(self.carrier_sense(), self.transmitting()).map(|(cs, t)| cs && t)
    }
}
