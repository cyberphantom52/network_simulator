use super::{MacAddr, ESCAPE, FLAG};
use crate::layers::{
    datalink::{EtherType, EthernetHeader},
    physical::physical::PhysicalLayer,
};
use crate::utils::crc::calculate_crc;

const TIMEOUT: u64 = 1000;

pub enum FrameType {
    Data,
    Ack,
    NAck,
    Invalid,
}

pub trait DataLink: PhysicalLayer {
    fn mac_addr(&self) -> &MacAddr;

    fn parse_frame(&self, raw: Vec<u8>) -> FrameType {
        if calculate_crc(&raw) != 0 {
            return FrameType::Invalid;
        }
        let header = EthernetHeader::from_be_bytes(&raw[1..15]);

        if header.src() == self.mac_addr() {
            return FrameType::Invalid;
        }

        if raw.len() == 19 {
            return FrameType::Ack;
        }

        FrameType::Data
    }

    fn byte_stuff(&self, data: &[u8]) -> Vec<u8> {
        let mut stuffed = Vec::with_capacity(data.len());
        for &byte in data {
            if byte == FLAG || byte == ESCAPE {
                stuffed.push(ESCAPE);
            }
            stuffed.push(byte);
        }
        stuffed
    }

    fn generate_frame(&self, data: &[u8]) -> Vec<u8> {
        let header = EthernetHeader::new(
            self.mac_addr().clone(),
            MacAddr::from("FF:FF:FF:FF:FF:FF"),
            EtherType::IPv4,
        );
        let mut frame = [[FLAG].as_ref(), &header.as_be_bytes(), data].concat();
        let fcs = calculate_crc(&frame).to_le_bytes();
        frame.append(&mut fcs.to_vec());
        frame.append(&mut [FLAG].to_vec());
        frame
    }

    fn send(&self, data: &[u8]) {
        let frame = self.generate_frame(self.byte_stuff(data).as_slice());
        let mut attempts = 0;
        self.tansmit(frame.clone(), None);

        while attempts < 10 {
            std::thread::sleep(std::time::Duration::from_millis(TIMEOUT));
            match self.get_from_pl() {
                Some(raw) => match self.parse_frame(raw) {
                    FrameType::Ack => {
                        return;
                    }
                    _ => (),
                },
                None => {
                    attempts += 1;
                    eprintln!("Frame/Ack Lost, re-attempting");
                    self.tansmit(frame.clone(), None);
                }
            }
        }

        eprintln!("Frame Dropped after 10 re-attempts");
    }

    fn ack(&self) {
      let ack = self.generate_frame(&[]);
      self.tansmit(ack, None);
    }

    fn get_from_pl(&self) -> Option<Vec<u8>> {
        let mut data = Vec::with_capacity(18);
        // defaut escape to true because the first byte is always a flag
        let mut escape = true;

        while let Some(byte) = self.receive() {
            if byte == ESCAPE && !escape {
                escape = true;
                continue;
            }

            if byte == FLAG && !escape {
                break;
            }

            data.push(byte);
            escape = false;
        }

        if data.len() == 0 {
            return None;
        }

        Some(data)
    }

    fn try_recv(&self) {
        match self.get_from_pl() {
            Some(raw_stream) => {
                match self.parse_frame(raw_stream) {
                    FrameType::Data => {
                        println!("Data frame recieved, sending back ACK");
                        self.ack();
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }
}
