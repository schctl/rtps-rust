use std::thread;
use std::time::Duration;

use domain::DomainConnection;
use entity::Entity;
use message::Message;
use participant::{RTPSParticipant, RemoteParticipant};

mod domain;
mod entity;
mod message;
mod participant;

fn sender() -> anyhow::Result<()> {
    let domain = DomainConnection::new()?;
    let mut participant = RTPSParticipant::new(domain);

    let hello_w = participant.new_writer("/hello");

    loop {
        participant.advertise_entities()?;

        thread::sleep(Duration::from_millis(2_000));
    }
}

fn listener() -> anyhow::Result<()> {
    let domain = DomainConnection::new()?;
    let mut participant = RTPSParticipant::new(domain);

    let hello_r = participant.new_reader("/hello");

    loop {
        participant.try_process_advertisements()?;
        println!("{participant:?}");

        thread::sleep(Duration::from_millis(1_000));
    }
}

fn main() {
    println!("Hello, world!");

    if std::env::args().nth(1).unwrap().to_lowercase() == "--client" {
        listener().unwrap();
    } else if std::env::args().nth(1).unwrap().to_lowercase() == "--server" {
        sender().unwrap();
    } else {
        eprintln!("Unknown opt");
        std::process::exit(-1);
    }
}
