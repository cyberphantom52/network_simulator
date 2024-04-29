use self::{datalink::MacAddr, physical::{physical::PhysicalLayer, port::Connection}};

pub mod datalink;
pub mod physical;

#[derive(PartialEq, Eq, Hash)]
/// Identifier for a device on the network
pub enum Identifier {
    Name(String),
    MacAddr(MacAddr)
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        match self {
            Identifier::Name(name) => name.clone(),
            Identifier::MacAddr(mac) => mac.to_string(),
        }
    }
}

enum ConnectionTarget<'a> {
    Device(Box<&'a mut dyn PhysicalLayer>),
    Connection(&'a Connection),
}
