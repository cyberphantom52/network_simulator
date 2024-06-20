use crate::layers::{
    AccessControl, ErrorControl, Link, MacAddr, PhysicalLayer, ReceiveState, ReceiveStatus,
    TransmitState, NIC,
};
use crate::run_async;
use crate::utils::Simulateable;
use futures::Future;
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, RwLock};

/// Mapping of MAaC addresses to the vlan and the interface they are connected to
type SwitchingTable = HashMap<MacAddr, (u8, usize)>;

const N_INTERFACES: usize = 8;

#[derive(Default)]
pub struct Switch {
    mac: MacAddr,
    switching_table: Mutex<SwitchingTable>,
    interfaces: [Arc<NIC>; N_INTERFACES],
    working_interface: RwLock<usize>,
    transmit_state: Mutex<TransmitState>,
    receive_state: Mutex<ReceiveState>,
}

impl PhysicalLayer for Switch {
    fn nic(&self) -> &NIC {
        let iface = run_async!(self.working_interface.read().await.clone());

        &self.interfaces[iface]
    }

    fn connect(&self, other: Arc<impl PhysicalLayer>) {
        if let Some(i) = self.available_interface() {
            let (one, two) = Link::connection();
            self.interfaces[i].set_connection(Some(one));
            other.nic().set_connection(Some(two));
        }
    }
}

impl ErrorControl for Switch {}
impl AccessControl for Switch {
    fn mac(&self) -> MacAddr {
        self.mac.clone()
    }

    fn transmit_state(&self) -> impl Future<Output = MutexGuard<TransmitState>> {
        self.transmit_state.lock()
    }

    fn receive_state(&self) -> impl Future<Output = MutexGuard<ReceiveState>> {
        self.receive_state.lock()
    }

    fn recognize_address(&self, _: &MacAddr) -> bool {
        true
    }
}

impl Switch {
    pub fn available_interface(&self) -> Option<usize> {
        for (i, iface) in self.interfaces.iter().enumerate() {
            if !iface.is_connected() {
                return Some(i);
            }
        }

        None
    }
}

impl Simulateable for Switch {
    async fn tick(&self) {
        let valid_ifaces = self
            .interfaces
            .iter()
            .enumerate()
            .filter(|(_, iface)| iface.is_connected())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        if let Some(&iface) = valid_ifaces
            .iter()
            .filter(|&i| self.interfaces[*i].is_receiving())
            .choose(&mut rand::thread_rng())
        {
            *self.working_interface.write().await = iface;
            match self.receive_frame().await {
                Ok(status) => match status {
                    ReceiveStatus::Ok(dest, src, type_len, data) => {
                        self.switching_table.lock().await.entry(src.clone()).or_insert((0, iface));

                        let entry = self.switching_table.lock().await.get(&dest).cloned();
                        if let Some((_, interface)) = entry {
                            *self.working_interface.write().await = interface;
                            self.transmit_frame(&dest, &src, type_len, data).await;
                        } else {
                            for i in valid_ifaces {
                                if i != iface {
                                    *self.working_interface.write().await = i;
                                    self.transmit_frame(&dest, &src, type_len, data.clone())
                                        .await;
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                },
                Err(e) => eprintln!("Error Switching frame: {:?}", e),
            }
        }

        println!("Table: {:?}", self.switching_table.lock().await);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[derive(Default)]
    struct TestDevice {
        nic: NIC,
        transmit_state: Mutex<TransmitState>,
        receive_state: Mutex<ReceiveState>,
    }

    impl PhysicalLayer for TestDevice {
        fn nic(&self) -> &NIC {
            &self.nic
        }
    }

    impl ErrorControl for TestDevice {}

    impl AccessControl for TestDevice {
        fn transmit_state(&self) -> impl Future<Output = MutexGuard<TransmitState>> {
            self.transmit_state.lock()
        }

        fn receive_state(&self) -> impl Future<Output = MutexGuard<ReceiveState>> {
            self.receive_state.lock()
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_switch() {
        let switch = Arc::new(Switch::default());
        let dev1 = Arc::new(TestDevice::default());
        let dev2 = Arc::new(TestDevice::default());
        let dev3 = Arc::new(TestDevice::default());

        let d1 = dev1.clone();
        let d2 = dev2.clone();
        let s1 = switch.clone();

        tokio::spawn(async move { s1.byte_transmitter().await });
        tokio::spawn(async move { d1.byte_transmitter().await });
        tokio::spawn(async move { d2.byte_transmitter().await });

        switch.connect(dev1.clone());
        switch.connect(dev2.clone());
        switch.connect(dev3.clone());

        let message = "Hello".bytes().collect::<Vec<u8>>();
        let len = message.len() as u16;
        println!(
            "D1 Transmit Status: {:?}",
            dev1.transmit_frame(&dev2.mac(), &dev1.mac(), len, message.clone())
                .await
        );
        for i in 0..50 {
            switch.tick().await;
        }
        println!("{:?}", dev2.receive_frame().await);

        println!(
            "D2 Transmit Status: {:?}",
            dev2.transmit_frame(&dev1.mac(), &dev2.mac(), len, message.clone())
                .await
        );
        for i in 0..50 {
            switch.tick().await;
        }
        println!("{:?}", dev1.receive_frame().await);
    }
}
