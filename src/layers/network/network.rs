use tokio::sync::Mutex;
use rand::Rng;
use std::{collections::HashMap};

use crate::layers::{AccessControl, ErrorControl, MacAddr, PhysicalLayer, ReceiveState, TransmitState, NIC};


pub type IpAddr = [u8; 4];

pub struct DhcpServer {
    nic: Mutex<NIC>,
    ip_table: HashMap<IpAddr, MacAddr>,
    transmit_state: Mutex<TransmitState>,
    receive_state: Mutex<ReceiveState>,
    ip_pool: Vec<IpAddr>,
}

impl DhcpServer {
    pub fn new() -> Self {
        let mut server = DhcpServer {
            nic: Default::default(),
            ip_table: HashMap::new(),
            transmit_state: Mutex::new(TransmitState::default()),
            receive_state: Mutex::new(ReceiveState::default()),
            ip_pool: Vec::new(),
        };
        server
    }

    pub fn dhcp(&mut self) -> IpAddr {
        loop {
            let ip: IpAddr = rand::thread_rng().gen();
            if !self.ip_table.contains_key(&ip) {
                let mac = MacAddr::default();
                self.ip_table.insert(ip, mac);
                return ip;
            }
        }
    }

    pub fn apipa(&self) -> IpAddr {
        let ip = [
            169,
            254,
            rand::thread_rng().gen_range(1..255),
            rand::thread_rng().gen_range(1..255),
        ];
        ip
    }

}

pub trait NetworkLayer: AccessControl {
    fn get_ip(&self,dhcpServer: &mut DhcpServer) -> IpAddr {
         if let ip = dhcpServer.dhcp() {
             ip
         } else {
             dhcpServer.apipa()
         }
    }
}

