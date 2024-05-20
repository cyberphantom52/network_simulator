use super::{header::TypeLen, MacAddr};
use crate::layers::physical::PhysicalLayer;

pub enum TransmitStatus {}
pub enum RecieveStatus {}

pub trait AccessControl: PhysicalLayer {
    fn backoff(&self, attempt: usize);
    fn watch_for_collision(&self);
    fn encapsulate_frame(
        &self,
        dest: &MacAddr,
        src: &MacAddr,
        type_len: TypeLen,
        frame: Vec<u8>,
    ) -> Vec<u8>;
    fn transmit_frame(
        &self,
        dest: &MacAddr,
        src: &MacAddr,
        type_len: TypeLen,
        frame: Vec<u8>,
    ) -> Result<TransmitStatus, TransmitStatus>;
    fn transmit_link_management(&self) -> Result<TransmitStatus, TransmitStatus>;
    fn start_transmit(&self);

    fn receive_frame(&self) -> Result<RecieveStatus, RecieveStatus>;
    fn decapsulate_frame(&self, frame: Vec<u8>) -> (MacAddr, MacAddr, u16, Vec<u8>);
    fn receive_link_management(&self) -> Result<RecieveStatus, RecieveStatus>;
    fn start_receive(&self);
}
