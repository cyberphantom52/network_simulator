use crate::{
    layers::{ConnectionMap, Identifier, Interface, PhysicalLayer},
    utils::Simulateable,
};
use futures::{future::join_all, StreamExt};
use rand::{distributions::Alphanumeric, Rng};

pub struct Hub {
    id: Identifier,
    interfaces: [Interface; 8],
    map: ConnectionMap,
}

impl Default for Hub {
    fn default() -> Self {
        let id = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect::<String>();
        Hub {
            id: Identifier::Name(format!("hub-{}", id)),
            interfaces: Default::default(),
            map: Default::default(),
        }
    }
}

impl PhysicalLayer for Hub {
    fn id(&self) -> &Identifier {
        &self.id
    }

    fn conn_map(&self) -> &ConnectionMap {
        &self.map
    }

    fn conn_map_mut(&mut self) -> &mut ConnectionMap {
        &mut self.map
    }

    fn interfaces(&self) -> &[Interface] {
        &self.interfaces
    }

    fn interfaces_mut(&mut self) -> &mut [Interface] {
        &mut self.interfaces
    }

    /// Receive a byte from a random connected interface
    async fn receive(&self, _: Option<usize>) -> Option<(u8, usize)> {
        use rand::seq::IteratorRandom;
        self.interfaces()
            .iter()
            .enumerate()
            .filter(|(_, inteface)| inteface.is_connected())
            .choose(&mut rand::thread_rng())
            .and_then(|(index, inteface)| inteface.recv().map(|byte| (byte, index)))
    }

    /// Broadcast a byte to all connected interfaces except the one with the given index
    async fn transmit(&self, byte: u8, exclude: Option<usize>) {
        join_all(
            self.interfaces()
                .iter()
                .enumerate()
                .filter(|(index, interface)| interface.is_connected() && exclude != Some(*index))
                .map(|(_, interface)| interface.send(byte)),
        )
        .await;
    }
}

impl Simulateable for Hub {
    async fn tick(&self) {
        if let Some((byte, port)) = self.receive(None).await {
            self.transmit(byte, Some(port)).await;
        }
    }
}
