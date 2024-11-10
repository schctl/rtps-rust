use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use domain::RTPSDomain;
use entity::Entity;
use message::EntityDiscovery;

mod domain;
mod entity;
mod message;
mod participant;

fn sender() -> anyhow::Result<()> {
    let domain = RTPSDomain::new()?;

    loop {
        domain.send_message_multicast(EntityDiscovery(Entity {
            id: 2,
            kind: entity::Type::Reader("/hello".to_owned()),
        }).into())?;

        thread::sleep(Duration::from_millis(500));
    }
}

fn listener() -> anyhow::Result<()> {
    let domain = RTPSDomain::new()?;

    loop {
        if let Ok(Some((addr, msg))) = domain.try_recv_message() {
            println!("{addr}: {msg:?}");
        }
    }
}

fn main() {
    println!("Hello, world!");

    if std::env::args().nth(1).unwrap().to_lowercase() == "--listener" {
        listener().unwrap();
    } else if std::env::args().nth(1).unwrap().to_lowercase() == "--server" {
        sender().unwrap();
    } else {
        eprintln!("Unknown opt");
        std::process::exit(-1);
    }
}
