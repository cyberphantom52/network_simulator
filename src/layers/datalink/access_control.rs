use crate::layers::physical::PhysicalLayer;

pub trait AccessControl: PhysicalLayer {
    fn is_transmit_allowed(&self) -> bool;
    fn is_transmission_complete(&self) -> bool;
}
