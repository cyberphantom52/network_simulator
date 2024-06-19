use network_simulator::{
    AccessControl, ErrorControl, PhysicalLayer, ReceiveState, ReceiveStatus, TransmitState, NIC,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
struct Device {
    nic: NIC,
    transmit_state: Mutex<TransmitState>,
    receive_state: Mutex<ReceiveState>,
}

impl PhysicalLayer for Device {
    fn nic(&self) -> &NIC {
        &self.nic
    }
}

impl ErrorControl for Device {}
impl AccessControl for Device {
    fn transmit_state(
        &self,
    ) -> impl futures::Future<Output = tokio::sync::MutexGuard<TransmitState>> {
        self.transmit_state.lock()
    }

    fn receive_state(
        &self,
    ) -> impl futures::Future<Output = tokio::sync::MutexGuard<ReceiveState>> {
        self.receive_state.lock()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_access_control() {
    let dev1 = Arc::new(Device::default());
    let dev2 = Arc::new(Device::default());

    let d1 = dev1.clone();
    tokio::spawn(async move { d1.byte_transmitter().await });

    dev1.connect(dev2.clone());

    let message = "Hello".bytes().collect::<Vec<u8>>();
    let len = message.len() as u16;
    let status = dev1
        .transmit_frame(&dev2.mac(), &dev1.mac(), len, message.clone())
        .await;
    assert!(status.is_ok());
    let status = dev2.receive_frame().await;
    assert!(status.is_ok());
    match status {
        Ok(ReceiveStatus::Ok(_, _, _, data)) => {
            assert_eq!(data, message);
        }
        _ => panic!("Error receiving frame"),
    }
}
