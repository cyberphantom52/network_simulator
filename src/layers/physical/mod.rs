mod link;
mod interface;
mod physical;

pub use physical::PhysicalLayer;

pub(self) type ConnectionMap = std::collections::HashMap<String, usize>;
