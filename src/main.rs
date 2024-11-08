use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::thread;
use std::time::Duration;

lazy_static::lazy_static! {
    static ref MULTICAST_GROUP: Ipv4Addr = "224.0.1.23".parse().unwrap();
    static ref INTERFACE: Ipv4Addr = "0.0.0.0".parse().unwrap();
    static ref LISTEN_PORT: u16 = 3589;
    static ref SEND_PORT: u16 = 3596;
}

fn make_socket(port: u16) -> anyhow::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddrV4::new(*INTERFACE, port))?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    Ok(socket)
}

fn sender() -> anyhow::Result<()> {
    let socket = make_socket(*SEND_PORT)?;
    
    socket.join_multicast_v4(&MULTICAST_GROUP, &INTERFACE)?;

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        socket.send_to(input.as_bytes(), SocketAddrV4::new(*INTERFACE, *LISTEN_PORT))?;
        thread::sleep(Duration::from_millis(500));
    }
}

fn listener() -> anyhow::Result<()> {
    let socket = make_socket(*LISTEN_PORT)?;

    socket.join_multicast_v4(&MULTICAST_GROUP, &INTERFACE)?;

    loop {
        let mut data_buf = [0; 256];

        match socket.recv_from(&mut data_buf) {
            Ok((len, addr)) => {
                println!(
                    "got data: {} from: {}",
                    String::from_utf8_lossy(&data_buf[..len]),
                    addr
                );
            }
            Err(err) => {
                match err.kind() {
                    _ => {}
                }
            }
        }
    }

    Ok(())
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
