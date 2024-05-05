mod interface;
mod link;
mod physical;

pub use interface::Interface;
pub use physical::PhysicalLayer;

pub type ConnectionMap = std::collections::HashMap<String, usize>;
