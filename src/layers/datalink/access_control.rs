use crate::layers::physical::PhysicalLayer;
use rand::Rng;

const MAX_ATTEMPTS: u8 = 15;
const P: f32 = 0.01;

pub trait AccessControl: PhysicalLayer {
    fn is_transmit_allowed(&self, interface_number: usize) -> bool {
        self.is_channel_idle(interface_number)
    }

    fn is_transmission_complete(&self) -> bool;

    fn send(&self, interface_number: usize, frame: Vec<u8>) {
        let mut attempt: u8 = 0;
        if self.is_transmit_allowed(interface_number) {
            for byte in &frame {
                self.transmit(*byte, Some(interface_number));
            }
            return;
        }
        while attempt < MAX_ATTEMPTS {
            if self.is_transmit_allowed(interface_number) {
                if get_probability() < P {
                    for byte in &frame {
                        self.transmit(*byte, Some(interface_number));
                    }
                    break;
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(get_backoff_time(attempt)));
                    attempt += 1;
                }
            }
        }
    }
}

fn get_probability() -> f32 {
    rand::thread_rng().gen()
}

fn get_backoff_time(attempt: u8) -> u64 {
    let max_backoff_time = 2u64.pow(attempt as u32);
    rand::thread_rng().gen_range(0..max_backoff_time)
}
