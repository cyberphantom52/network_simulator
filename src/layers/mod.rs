use self::physical::{physical::PhysicalLayer, port::Connection};

pub mod datalink;
pub mod physical;

#[derive(PartialEq, Eq, Hash)]
/// Identifier for a device on the network
enum Identifier {
    Name(String),
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        match self {
            Identifier::Name(name) => name.clone(),
        }
    }
}

enum ConnectionTarget<'a> {
    Device(Box<&'a mut dyn PhysicalLayer>),
    Connection(&'a Connection),
}
