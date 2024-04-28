use layers::{datalink::{datalink::DataLink, MacAddr}, physical::{physical::PhysicalLayer, port::{Connection, PhysicalPort}, ConnectionMap}, ConnectionTarget, Identifier};

mod layers;
mod utils;

use std::thread;

struct Device {
    id: Identifier,
    ports: [PhysicalPort; 5],
    map: ConnectionMap,
}

impl Device {

    pub fn new() -> Self {
        Device {
            id: Identifier::MacAddr(MacAddr::new()),
            ports:
                [
                    PhysicalPort::default(),
                    PhysicalPort::default(),
                    PhysicalPort::default(),
                    PhysicalPort::default(),
                    PhysicalPort::default(),
                ],
            map: ConnectionMap::new(),
        }
    }
}

impl PhysicalLayer for Device {
    fn id(&self) -> &Identifier {
        &self.id
    }

    fn ports(&self) -> &[PhysicalPort] {
        &self.ports
    }

    fn ports_mut(&mut self) -> &mut [PhysicalPort] {
        &mut self.ports
    }

    fn conn_map(&self) -> &ConnectionMap {
        &self.map
    }

    fn conn_map_mut(&mut self) -> &mut ConnectionMap {
        &mut self.map
    }
}

impl DataLink for Device {
    fn mac_addr(&self) -> &MacAddr {
        match &self.id {
            Identifier::MacAddr(mac) => mac,
            _ => panic!("Device doesn't have a MAC address")
        }
    }
}

fn main() {
    let mut devices = vec![Device::new(), Device::new(), Device::new()];
    let bus = Connection::new();

    for i in 0..devices.len() {
        devices[i].connect(ConnectionTarget::Connection(&bus));
    }

    thread::scope(|scope| {
       scope.spawn(||{
           loop {
            devices[1].try_recv();
           }
       });
       scope.spawn(|| {
           loop {
            devices[2].try_recv();
           }
       });
       scope.spawn(|| {
           devices[0].send("Hello".as_bytes());
           loop {}
       });
    });

}
