mod crc;
pub use crc::calculate_crc;

#[macro_export]
macro_rules! arc_mutex {
    ($data:expr) => {
        std::sync::Arc::new(Mutex::new($data))
    };
}

pub trait Simulateable {
    async fn tick(&self);
}
