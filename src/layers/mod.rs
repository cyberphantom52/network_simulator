mod physical;
mod datalink;

use physical::PhysicalLayer;
use datalink::DataLinkLayer;

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
