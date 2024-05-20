mod physical;
mod datalink;

pub use physical::{PhysicalLayer, ConnectionMap, Interface};

#[derive(Debug, Clone)]
pub enum Identifier {
    Name(String),
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        match self {
            Identifier::Name(name) => name.clone(),
        }
    }
}
