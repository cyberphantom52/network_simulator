mod crc;

#[macro_export]
macro_rules! arc_mutex {
    ($data:expr) => {
        Arc::new(Mutex::new($data))
    };
}

pub trait Simulateable {
    fn tick(&mut self);
}
