use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::ops::Range;
use std::time::Duration;

use socket2::{Domain, Protocol, SockAddr, Socket};

use anyhow::bail;

use crate::message::Message;

lazy_static::lazy_static! {
    // multicast interfaces
    static ref MULTICAST_GROUP: Ipv4Addr = [224, 0, 0, 23].into();
    static ref MULTICAST_INTF: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
    static ref MULTICAST_PORT: u16 = 7399;

    // unicast interfaces
    static ref INTERFACE: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
    static ref PORT_RANGE: Range<u16> = 7400..8000;
}

fn addr<A: Into<Ipv4Addr>>(ip_v4: A, port: u16) -> SockAddr {
    SocketAddrV4::new(ip_v4.into(), port).into()
}

#[derive(Debug)]
pub struct DomainConnection {
    socket: Socket,
    discovery_socket: Socket,
}

impl DomainConnection {
    fn make_socket() -> anyhow::Result<Socket> {
        let socket = Socket::new(Domain::IPV4, socket2::Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        Ok(socket)
    }

    fn setup_sock_unicast() -> anyhow::Result<Socket> {
        let socket = Self::make_socket()?;

        for i in PORT_RANGE.clone() {
            if socket.bind(&addr(*INTERFACE, i)).is_ok() {
                break;
            }
        }

        let _ = socket.local_addr()?;

        Ok(socket)
    }

    fn setup_sock_multicast() -> anyhow::Result<Socket> {
        let socket = Self::make_socket()?;
        let multicast_interface = addr(*MULTICAST_INTF, *MULTICAST_PORT);

        // socket should be READ-only
        socket.set_reuse_port(true)?;
        socket.bind(&multicast_interface)?;
        socket.join_multicast_v4(&MULTICAST_GROUP, &MULTICAST_INTF)?;

        Ok(socket)
    }

    pub fn new() -> anyhow::Result<Self> {
        let socket = Self::setup_sock_unicast()?;
        let discovery_socket = Self::setup_sock_multicast()?;

        Ok(Self {
            socket,
            discovery_socket,
        })
    }

    pub fn send_message<T: Into<SockAddr>>(&self, msg: Message, to: T) -> anyhow::Result<()> {
        self.socket
            .send_to(&postcard::to_vec::<Message, 128>(&msg)?, &to.into())?;
        Ok(())
    }

    pub fn send_message_discovery(&self, msg: Message) -> anyhow::Result<()> {
        self.send_message(msg, addr(*MULTICAST_GROUP, *MULTICAST_PORT))
    }

    fn try_recv_message_intl(
        &self,
        socket: &Socket,
    ) -> anyhow::Result<Option<(SocketAddr, Message)>> {
        let mut data = [const { MaybeUninit::uninit() }; 128];

        match socket.recv_from(&mut data) {
            Ok((bytes, addr)) => {
                let data =
                    unsafe { core::mem::transmute::<[MaybeUninit<u8>; 128], [u8; 128]>(data) };

                if let Ok(msg) = postcard::from_bytes::<Message>(&data) {
                    return Ok(Some((addr.as_socket().unwrap(), msg)));
                } else {
                    let addr = addr.as_socket_ipv4().unwrap();
                    tracing::error!("unable to process {bytes} bytes from {addr}. dropping.");
                }
            }
            Err(err) => {
                bail!(err);
            }
        }

        Ok(None)
    }

    pub fn try_recv_message(&self) -> anyhow::Result<Option<(SocketAddr, Message)>> {
        self.try_recv_message_intl(&self.socket)
    }

    pub fn try_recv_message_discovery(&self) -> anyhow::Result<Option<(SocketAddr, Message)>> {
        self.try_recv_message_intl(&self.discovery_socket)
    }
}
