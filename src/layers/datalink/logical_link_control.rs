use super::media_access_control::AccessControl;
use crate::layers::physical::PhysicalLayer;

pub trait LogicalLinkControl: PhysicalLayer + AccessControl {
    async fn frame_transmitter();
    async fn frame_reciever();
}
