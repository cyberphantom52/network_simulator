mod crc;
pub use crc::calculate_crc;

#[macro_export]
macro_rules! arc_mutex {
    ($data:expr) => {
        std::sync::Arc::new(Mutex::new($data))
    };
}

#[macro_export]
macro_rules! run_async {
    ($data:expr) => {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move { $data })
        })
    };
}

pub trait Simulateable {
    async fn tick(&self);
}
