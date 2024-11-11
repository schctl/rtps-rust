use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use domain::DomainConnection;
use participant::RTPSParticipant;

mod domain;
mod entity;
mod message;
mod participant;

fn sender() -> anyhow::Result<()> {
    let domain = DomainConnection::new()?;
    let mut participant = RTPSParticipant::new(domain);

    let hello_w = participant.new_writer("/hello");

    let (tx, rx) = channel();

    std::thread::spawn(move || {
        let mut buf = String::new();
        let io = std::io::stdin();

        loop {
            if io.read_line(&mut buf).is_ok() {
                if let Err(e) = tx.send(buf.clone()) {
                    tracing::error!("{e}");
                }

                buf.clear();
            }
        }
    });

    loop {
        participant.advertise_entities()?;
        participant.try_process_advertisements()?;
        participant.process_all()?;

        if let Ok(line) = rx.try_recv() {
            hello_w.lock().unwrap().write(line);
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn listener() -> anyhow::Result<()> {
    let domain = DomainConnection::new()?;
    let mut participant = RTPSParticipant::new(domain);

    let hello_r = participant.new_reader("/hello");

    loop {
        participant.advertise_entities()?;
        participant.try_process_advertisements()?;
        participant.process_all()?;

        for m in hello_r.lock().unwrap().pop() {
            println!("{m:?}");
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    if std::env::args().nth(1).unwrap().to_lowercase() == "--client" {
        listener().unwrap();
    } else if std::env::args().nth(1).unwrap().to_lowercase() == "--server" {
        sender().unwrap();
    } else {
        eprintln!("Unknown opt");
        std::process::exit(-1);
    }
}
