use crate::layers::physical::PhysicalLayer;
use futures::Future;
use tokio::sync::MutexGuard;
use super::{
    error_control::ErrorControl,
    header::{EthernetHeader, TypeLen},
    MacAddr,
};

const FLAG: u8 = 0b10101011;

/// Size of the slot in byte times
const SLOT_SIZE: usize = 512;

/// Interframe space
const IFS: usize = 12;

const CRC_SIZE: usize = 4;
const ETHERNET_HEADER_SIZE: usize = 14;

// Frame sizes
const MIN_FRAME_SIZE: usize = 64;
const MAX_BASIC_FRAME_SIZE: usize = 1518;
const MAX_ENVELOPE_FRAME_SIZE: usize = 2000;

const MIN_TYPE_VAL: u16 = 1536;

const EXTEND: bool = (SLOT_SIZE - MIN_FRAME_SIZE) > 0;

const MAX_ATTEMPTS: usize = 16;
const MAX_BACKOFF: usize = 10;

const HALF_DUPLEX: bool = true;

#[derive(Debug, Clone)]
pub enum TransmitStatus {
    Ok,
    ExcessiveCollisions,
}

pub struct TransmitState {
    outgoing_frame: Vec<u8>,
    attempts: usize,
    current_transmit_byte: usize,
    last_transmit_byte: usize,
    transmit_succeeding: bool,
    new_collision: bool,
}

impl Default for TransmitState {
    fn default() -> Self {
        TransmitState {
            outgoing_frame: Vec::new(),
            attempts: 0,
            current_transmit_byte: 0,
            last_transmit_byte: 0,
            transmit_succeeding: false,
            new_collision: false,
        }
    }
}

pub enum RecieveStatus {
    Ok,
    LengthError,
    FrameCheckError,
    AlignmentError,
}

pub trait AccessControl: PhysicalLayer + ErrorControl {
    fn transmit_state(&self) -> impl Future<Output = MutexGuard<TransmitState>>;

    /// Backoff for a random number time slots specified by the attempt number
    ///
    /// Uses the exponential backoff algorithm
    async fn backoff(&self, attempt: usize) {
        use rand::Rng;
        use std::time::Duration;
        let max_backoff = 2usize.pow(attempt.min(MAX_BACKOFF) as u32);
        let backoff = (rand::thread_rng().gen_range(0..max_backoff) * SLOT_SIZE) as u64;
        tokio::time::sleep(Duration::from_millis(backoff)).await;
    }

    /// An async process that watches for collisions on the network
    /// and sets the collision flag if a collision is detected
    async fn watch_for_collision(&self) {
        while self.nic().await.transmitting() {
            let mut state = self.transmit_state().await;
            if state.transmit_succeeding && self.collision_detect().await {
                state.new_collision = true;
                state.transmit_succeeding = false;
            }
        }
    }

    /// Encapsulates a frame with the Ethernet header and frame check sequence
    ///
    /// Also pads the frame to make sure the it meets the minimum frame size requirement
    fn encapsulate_frame(&self, dest: &MacAddr, src: &MacAddr, type_len: TypeLen, frame: Vec<u8>) -> Vec<u8> {
        let pad_size = MIN_FRAME_SIZE.saturating_sub(ETHERNET_HEADER_SIZE + CRC_SIZE + frame.len());
        let header = EthernetHeader::new(src, dest, type_len);
        let mut encapsulated_frame = [
            &[FLAG],
            header.to_be_bytes().as_ref(),
            frame.as_ref(),
            vec![0b01010101; pad_size].as_ref(),
        ].concat();
        let fcs = Self::fcs(&encapsulated_frame);
        encapsulated_frame.extend(fcs.to_be_bytes());

        encapsulated_frame
    }

    /// Decapsulates a frame and returns the destination, source, type/length, and data
    fn decapsulate_frame(&self, frame: Vec<u8>) -> (MacAddr, MacAddr, u16, Vec<u8>) {
        let mut dest = [0; 6];
        let mut src = [0; 6];
        dest.copy_from_slice(&frame[0..6]);
        src.copy_from_slice(&frame[6..12]);
        let type_len = u16::from_be_bytes([frame[13], frame[14]]);

        let data = match type_len >= MIN_TYPE_VAL {
            true => frame[15..frame.len() - CRC_SIZE].to_vec(),
            false => frame[15..(15 + type_len as usize)].to_vec(),
        };

        return (MacAddr::from(dest), MacAddr::from(src), type_len, data);
    }

    /// An async process that is continuously running and transmits bytes on the network
    async fn byte_transmitter(&self) {
        loop {
            if self.nic().await.transmitting() {
                loop {
                    while self.nic().await.transmitting() {
                        let mut state = self.transmit_state().await;
                        self.transmit(state.outgoing_frame[state.current_transmit_byte]).await;
                        if state.new_collision {
                            state.current_transmit_byte = 1;
                            state.new_collision = false;
                            self.nic().await.set_transmitting(false);
                        } else {
                            state.current_transmit_byte += 1;
                            self.nic().await.set_transmitting(state.current_transmit_byte < state.last_transmit_byte);
                        }
                    }
                }
            }
        }
    }

    /// The interface for MAC Client by which it can transmit a frame
    ///
    /// Uses the CSMA/CD algorithm to transmit the frame
    async fn transmit_frame(
        &self,
        dest: &MacAddr,
        src: &MacAddr,
        type_len: TypeLen,
        frame: Vec<u8>,
    ) -> Result<TransmitStatus, TransmitStatus> {
        let mut state = self.transmit_state().await;
        state.outgoing_frame = self.encapsulate_frame(dest, src, type_len, frame);
        state.attempts = 0;
        state.transmit_succeeding = false;

        while state.attempts < MAX_ATTEMPTS && !state.transmit_succeeding {
            if state.attempts > 0 {
                self.backoff(state.attempts).await;
            }

            state.current_transmit_byte = 1;
            state.last_transmit_byte = state.outgoing_frame.len();
            state.transmit_succeeding = true;
            self.nic().await.set_transmitting(true);

            drop(state);
            self.watch_for_collision().await;
            state = self.transmit_state().await;

            state.attempts += 1;
        }

        if state.transmit_succeeding {
            return Ok(TransmitStatus::Ok);
        }

        Err(TransmitStatus::ExcessiveCollisions)
    }
}
