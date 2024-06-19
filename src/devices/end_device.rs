use tokio::sync::Mutex;
use crate::layers::{AccessControl, DhcpServer, ErrorControl, IpAddr, NetworkLayer, PhysicalLayer};
use crate::layers::{ReceiveState, TransmitState, NIC};


pub struct EndDevice {
    ip: IpAddr,
    nic: Mutex<NIC>,
    transmit_state: Mutex<TransmitState>,
    receive_state: Mutex<ReceiveState>,
}

impl EndDevice {
    pub fn new() -> Self {
        let device = EndDevice {
            ip: [0; 4],
            nic: Default::default(),
            transmit_state: Mutex::new(TransmitState::default()),
            receive_state: Mutex::new(ReceiveState::default()),
        };
        device
    }
}

impl NetworkLayer for EndDevice {
    fn get_ip(&self,dhcp_server: &mut DhcpServer) -> IpAddr {
        let ip = dhcp_server.dhcp();
        ip
    }
}

impl PhysicalLayer for EndDevice {
   fn nic(&self) -> impl futures::prelude::Future<Output = tokio::sync::MutexGuard<NIC>> {
        self.nic.lock()
   }
}

impl AccessControl for EndDevice {
   fn transmit_state(&self) -> impl futures::prelude::Future<Output = tokio::sync::MutexGuard<TransmitState>> {
        self.transmit_state.lock()
    }

    fn receive_state(&self) -> impl futures::prelude::Future<Output = tokio::sync::MutexGuard<ReceiveState>> {
        self.receive_state.lock()
   }
}

impl ErrorControl for EndDevice {
  
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_end_device() {
        let mut dhcp_server = DhcpServer::new();
        let device = EndDevice::new();
        let ip = device.get_ip(&mut dhcp_server);
    }
}

