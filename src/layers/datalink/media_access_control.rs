use super::{
    error_control::ErrorControl,
    header::{EthernetHeader, TypeLen},
    MacAddr,
};

use crate::layers::physical::PhysicalLayer;
use futures::{Future, FutureExt};
use tokio::sync::MutexGuard;

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

#[derive(Default)]
pub struct TransmitState {
    outgoing_frame: Vec<u8>,
    new_collision: bool,
}

#[derive(Default, Debug, Clone)]
pub struct ReceiveState {
    incoming_frame: Vec<u8>,
    receiving: bool,
    receive_succeeeding: bool,
    valid_length: bool,
}

#[derive(Debug, Clone)]
pub enum ReceiveStatus {
    Ok(MacAddr, MacAddr, u16, Vec<u8>),
    FrameTooLong,
    FrameCheckError,
}

pub trait AccessControl: PhysicalLayer + ErrorControl {
    fn transmit_state(&self) -> impl Future<Output = MutexGuard<TransmitState>>;
    fn receive_state(&self) -> impl Future<Output = MutexGuard<ReceiveState>>;

    fn mac(&self) -> MacAddr {
        self.nic().mac()
    }

    /// Encapsulates a frame with the Ethernet header and frame check sequence
    ///
    /// Also pads the frame to make sure the it meets the minimum frame size requirement
    fn encapsulate_frame(
        &self,
        dest: &MacAddr,
        src: &MacAddr,
        type_len: TypeLen,
        frame: Vec<u8>,
    ) -> Vec<u8> {
        let pad_size = MIN_FRAME_SIZE.saturating_sub(ETHERNET_HEADER_SIZE + CRC_SIZE + frame.len());
        let header = EthernetHeader::new(src, dest, type_len);
        let mut encapsulated_frame = [
            &[FLAG],
            header.to_be_bytes().as_ref(),
            frame.as_ref(),
            vec![0b01010101; pad_size].as_ref(),
        ]
        .concat();
        let fcs = Self::fcs(&encapsulated_frame);
        encapsulated_frame.extend(fcs.to_le_bytes());
        encapsulated_frame
    }

    fn recognize_address(&self, destination: &MacAddr) -> bool {
        // TODO: Promiscuous and multicast mode
        destination == &MacAddr::broadcast() || destination == &self.mac()
    }

    /// Decapsulates a frame and returns the destination, source, type/length, and data
    async fn decapsulate_frame(&self) -> Result<ReceiveStatus, ReceiveStatus> {
        fn remove_padding(type_len: TypeLen, data: Vec<u8>) -> Vec<u8> {
            if type_len >= MIN_TYPE_VAL {
                return data;
            }

            if type_len as usize <= MAX_BASIC_FRAME_SIZE - 18 {
                if data.len() != type_len as usize {
                    return data[0..type_len as usize].to_vec();
                }
            }

            return data;
        }

        // TODO: If we use Option, we can use .take() to get the frame and set it to None
        let mut frame = self.receive_state().await.incoming_frame.clone();
        if Self::fcs(&frame) != 0 {
            return Err(ReceiveStatus::FrameCheckError);
        }

        let mut dest = [0; 6];
        dest.copy_from_slice(&frame[1..7]);

        self.receive_state().await.receive_succeeeding =
            self.recognize_address(&MacAddr::from(dest));
        if self.receive_state().await.receive_succeeeding {
            let mut src = [0; 6];
            src.copy_from_slice(&frame[7..13]);
            let type_len = u16::from_be_bytes([frame[13], frame[14]]);
            frame.drain(..15);
            let data = remove_padding(type_len, frame);
            if data.len() > MAX_ENVELOPE_FRAME_SIZE {
                return Err(ReceiveStatus::FrameTooLong);
            }
            return Ok(ReceiveStatus::Ok(
                MacAddr::from(dest),
                MacAddr::from(src),
                type_len,
                data,
            ));
        }

        Err(ReceiveStatus::FrameCheckError)
    }

    /// An async process that is continuously running and transmits bytes on the network
    async fn byte_transmitter(&self) {
        let mut transmitted_byte = 0;
        while self.transmitting() {
            let mut state = self.transmit_state().await;
            self.transmit(state.outgoing_frame[transmitted_byte]).await;
            if state.new_collision {
                state.new_collision = false;
                self.nic().set_transmitting(false);
            } else {
                transmitted_byte += 1;
                self.nic()
                    .set_transmitting(transmitted_byte < state.outgoing_frame.len());
            }
        }
    }

    /// An async process that watches for collisions on the network
    /// and sets the collision flag if a collision is detected
    async fn watch_for_collision(&self, transmit_succeeding: &mut bool) {
        while self.transmitting() {
            if *transmit_succeeding && self.collision_detect() {
                self.transmit_state()
                    .map(|mut state| state.new_collision = true)
                    .await;
                *transmit_succeeding = false;
                break;
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
        async fn backoff(attempt: usize) {
            use rand::Rng;
            use std::time::Duration;
            let max_backoff = 2usize.pow(attempt.min(MAX_BACKOFF) as u32);
            let backoff = (rand::thread_rng().gen_range(0..max_backoff) * SLOT_SIZE) as u64;
            tokio::time::sleep(Duration::from_millis(backoff)).await;
        }

        let mut attempts = 0;
        let mut transmit_succeeding = false;
        self.transmit_state()
            .map(|mut state| {
                state.outgoing_frame = self.encapsulate_frame(dest, src, type_len, frame)
            })
            .await;

        while attempts < MAX_ATTEMPTS && !transmit_succeeding {
            if attempts > 0 {
                backoff(attempts).await;
            }

            transmit_succeeding = true;
            /* This start the actual transmission */
            self.nic().set_transmitting(true);

            self.watch_for_collision(&mut transmit_succeeding).await;

            attempts += 1;
        }

        if transmit_succeeding {
            return Ok(TransmitStatus::Ok);
        }

        Err(TransmitStatus::ExcessiveCollisions)
    }

    async fn receive_frame(&self) -> Result<ReceiveStatus, ReceiveStatus> {
        let mut result = Err(ReceiveStatus::FrameCheckError);
        while !self.receive_state().await.receive_succeeeding {
            while !self.receive_state().await.receive_succeeeding {
                self.receive_state()
                    .map(|mut state| {
                        state.receiving = true;
                        state.receive_succeeeding = true;
                    })
                    .await;

                if self.receive_state().await.receiving {
                    let mut frame = Vec::new();
                    while self.carrier_sense() {
                        if let Some(byte) = self.receive().await {
                            frame.push(byte);
                        }
                    }

                    self.receive_state()
                        .map(|mut state| {
                            state.incoming_frame = frame;
                            state.receiving = false;
                            state.receive_succeeeding = true;
                        })
                        .await;
                }

                self.receive_state()
                    .map(|mut state| {
                        let frame_size = state.incoming_frame.len();
                        state.receive_succeeeding =
                            state.receive_succeeeding && frame_size >= MIN_FRAME_SIZE;
                    })
                    .await;
            }
            result = self.decapsulate_frame().await;
        }

        self.receive_state().await.receive_succeeeding = false;
        return result;
    }
}
