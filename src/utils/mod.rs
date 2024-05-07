mod crc;

pub trait Simulateable {
    fn tick(&mut self);
}
