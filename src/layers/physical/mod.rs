use self::port::PortId;
use std::collections::HashMap;

pub mod physical;
pub mod port;

/// A map of connections
pub type ConnectionMap = HashMap<String, PortId>;
