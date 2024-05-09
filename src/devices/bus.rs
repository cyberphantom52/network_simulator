use super::hub::Hub;
use crate::{
    layers::{ConnectionMap, Identifier, Interface, PhysicalLayer},
    utils::Simulateable,
};
use futures::{future::join_all, StreamExt};
use rand::{distributions::Alphanumeric, Rng};

const N_JUNC: usize = 5;

pub struct Bus {
    id: Identifier,
    junctions: [Hub; N_JUNC],
    map: ConnectionMap,
}

impl Default for Bus {
    fn default() -> Self {
        let id = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect::<String>();
        Bus {
            id: Identifier::Name(format!("bus-{}", id)),
            junctions: Default::default(),
            map: ConnectionMap::default(),
        }
    }
}

impl Bus {
    fn index(number: usize) -> (usize, usize) {
        (number >> 16, number & 0x0000_FFFF)
    }
}

impl PhysicalLayer for Bus {
    fn id(&self) -> &Identifier {
        &self.id
    }

    fn conn_map(&self) -> &ConnectionMap {
        &self.map
    }

    fn conn_map_mut(&mut self) -> &mut ConnectionMap {
        &mut self.map
    }

    fn interface(&self, number: usize) -> &Interface {
        let (index, junction) = Self::index(number);
        self.junctions[junction].interface(index)
    }

    fn interface_mut(&mut self, number: usize) -> &mut Interface {
        let (index, junction) = Self::index(number);
        self.junctions[junction].interface_mut(index)
    }

    fn interfaces(&self) -> &[Interface] {
        unimplemented!("Getting interfaces is not supported for Bus")
    }

    fn interfaces_mut(&mut self) -> &mut [Interface] {
        unimplemented!("Getting interfaces is not supported for Bus")
    }

    fn availabe_interface(&self) -> Option<usize> {
        let junction = self
            .junctions
            .iter()
            .position(|j| j.availabe_interface().is_some())?;
        let index = self.junctions[junction].availabe_interface()?;
        Some((index << 16) | junction)
    }
}

impl Simulateable for Bus {
    async fn tick(&self) {
        join_all(self.junctions.iter().map(|j| j.tick())).await;
    }
}
