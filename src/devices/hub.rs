use crate::layers::{ConnectionMap, Identifier, Interface, PhysicalLayer};
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
    fn receive(&self, _: Option<usize>) -> Option<(u8, usize)> {
        use rand::seq::IteratorRandom;
        self.interfaces()
            .iter()
            .enumerate()
            .filter(|(_, inteface)| inteface.is_connected())
            .choose(&mut rand::thread_rng())
            .and_then(|(index, inteface)| inteface.recv().map(|byte| (byte, index)))
    }

    /// Broadcast a byte to all connected interfaces except the one with the given index
    fn transmit(&self, byte: u8, exclude: Option<usize>) {
        self.interfaces()
            .iter()
            .enumerate()
            .filter(|(index, interface)| interface.is_connected() && exclude != Some(*index))
            .for_each(|(_, interface)| {
                interface.send(byte);
            });
    }
}

impl Hub {
    pub async fn main_loop(&mut self) -> ! {
        loop {
            if let Some((byte, port)) = self.receive(None) {
                self.transmit(byte, Some(port));
            }
        }
    }
}
