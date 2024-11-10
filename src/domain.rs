use std::time::Duration;
use std::ops::Range;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};

use anyhow::bail;

use crate::message::Message;

lazy_static::lazy_static! {
    static ref MULTICAST_GROUP: Ipv4Addr = "224.0.0.23".parse().unwrap();
    static ref INTERFACE: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
    static ref PORT_RANGE: Range<u16> = 7400..8000;
}

pub struct RTPSDomain {
    socket: UdpSocket,
}

impl RTPSDomain {
    pub fn new() -> anyhow::Result<Self> {
        let mut socket = None;

        for i in PORT_RANGE.clone() {
            if let Ok(s) = UdpSocket::bind(SocketAddrV4::new(*INTERFACE, i)) {
                socket = Some(s);
                break;
            }
        }

        let socket = socket.unwrap(); // Eh

        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        socket.join_multicast_v4(&MULTICAST_GROUP, &INTERFACE)?;

        Ok(Self {
            socket
        })
    }

    pub fn socket_ref(&self) -> &UdpSocket {
        &self.socket
    }

    pub fn send_message<T: ToSocketAddrs>(&self, msg: Message, to: T) -> anyhow::Result<()> {
        self.socket.send_to(&postcard::to_vec::<Message, 128>(&msg)?, to).unwrap();
        Ok(())
    }

    pub fn send_message_multicast(&self, msg: Message, port: u16) -> anyhow::Result<()> {
        self.send_message(msg, SocketAddrV4::new(*INTERFACE, port))
    }

    pub fn try_recv_message(&self) -> anyhow::Result<Option<(SocketAddr, Message)>> {
        let mut data = [0; 128];
    
        match self.socket.recv_from(&mut data) {
            Ok((_, addr)) => {
                if let Ok(msg) = postcard::from_bytes::<Message>(&data) {
                    return Ok(Some((addr, msg)));
                }
            }
            Err(err) => {
                bail!(err);
            }
        }

        Ok(None)
    }
}
