use crate::layers::PhysicalLayer;

use super::{AccessControl, ErrorControl, FlowControl, LogicalLinkControl};

pub trait DataLinkLayer:
    PhysicalLayer + AccessControl + ErrorControl + FlowControl + LogicalLinkControl
{
}
