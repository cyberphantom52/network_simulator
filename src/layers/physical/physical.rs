use super::Link;
use crate::layers::NIC;
use futures::{future::join, Future, FutureExt};
use std::sync::Arc;
use tokio::sync::MutexGuard;

pub trait PhysicalLayer {
    fn nic(&self) -> impl Future<Output = MutexGuard<NIC>>;

    async fn connect(&self, other: Arc<impl PhysicalLayer>) {
        let (one, two) = Link::connection();
        self.nic().await.set_connection(Some(one));
        other.nic().await.set_connection(Some(two));
    }

    async fn disconnect(&mut self) {
        self.nic().await.set_connection(None);
    }

    async fn transmit(&self, byte: u8) {
        self.nic().await.transmit(byte);
    }

    async fn receive(&self) -> Option<u8> {
        self.nic().await.recieve()
    }

    fn carrier_sense(&self) -> impl Future<Output = bool> {
        self.nic()
            .map(|f| f.connection().map_or(false, |conn| conn.is_recieving()))
    }

    fn collision_detect(&self) -> impl Future<Output = bool> {
        let carrier_sense = self.carrier_sense();
        let transmitting = self.nic().map(|f| f.transmitting());
        join(carrier_sense, transmitting).map(|(cs, t)| cs && t)
    }
}
