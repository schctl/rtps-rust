use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use socket2::SockAddr;

use crate::domain::DomainConnection;
use crate::entity::{self, Entity, ReaderState, SharedReaderState, SharedWriterState, WriterState};
use crate::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteParticipant {
    pub entities: Vec<Entity>,
}

#[derive(Debug)]
pub struct RTPSParticipant {
    domain: DomainConnection,
    remote_participants: HashMap<SockAddr, RemoteParticipant>,
    writers: HashMap<Entity, SharedWriterState>,
    readers: HashMap<Entity, SharedReaderState>,
    last_clear_time: Instant,
}

impl RTPSParticipant {
    pub fn new(domain: DomainConnection) -> Self {
        Self {
            domain,
            remote_participants: HashMap::new(),
            writers: HashMap::new(),
            readers: HashMap::new(),
            last_clear_time: Instant::now(),
        }
    }

    pub fn new_writer<T: ToString>(&mut self, topic: T) -> SharedWriterState {
        let state = WriterState::new(topic.to_string().clone());
        self.writers
            .insert(Entity::new_writer(topic), state.clone());
        state
    }

    pub fn new_reader<T: ToString>(&mut self, topic: T) -> SharedReaderState {
        let state = ReaderState::new();
        self.readers
            .insert(Entity::new_reader(topic), state.clone());
        state
    }

    pub fn advertise_entities(&mut self) -> anyhow::Result<()> {
        let entities = self
            .writers
            .keys()
            .chain(self.readers.keys())
            .map(Clone::clone)
            .collect::<Vec<_>>();

        self.domain
            .send_message_discovery(RemoteParticipant { entities }.into())
    }

    #[allow(irrefutable_let_patterns)]
    pub fn try_process_advertisements(&mut self) -> anyhow::Result<()> {
        if Instant::now() - self.last_clear_time > Duration::from_secs(5) {
            self.remote_participants.clear();
        }

        if let Ok(Some((addr, msg))) = self.domain.try_recv_message_discovery() {
            if let Message::ParticipantRegister(pr) = msg {
                self.remote_participants.insert(addr.into(), pr);
            }
        }

        Ok(())
    }

    fn process_writers(&mut self) -> anyhow::Result<()> {
        let mut command_queue: HashMap<&SockAddr, Vec<Message>> = HashMap::new();

        for writer in &self.writers {
            let mut state = writer.1.lock().unwrap();

            if state.message_cache.is_empty() {
                continue;
            }

            // not the most efficient but whatever
            if let Some((s, _)) = self.remote_participants.iter().find(|(_, p)| {
                p.entities
                    .iter()
                    .find(|entity| &entity.reverse() == writer.0)
                    .is_some()
            }) {
                if let Some(cache) = command_queue.get_mut(s) {
                    cache.extend_from_slice(&state.message_cache);
                    state.clear();
                } else {
                    command_queue.insert(s, state.message_cache.clone());
                    state.clear();
                }
            }
        }

        for (sock, message) in command_queue {
            for msg in message {
                let sock_v4 = sock.as_socket_ipv4().unwrap();

                match self.domain.send_message(msg, sock.clone()) {
                    Ok(_) => tracing::trace!("send message to {sock_v4}"),
                    Err(e) => tracing::info!("error sending to {sock_v4}: {e}"),
                }
            }
        }

        Ok(())
    }

    fn process_readers(&mut self) -> anyhow::Result<()> {
        while let Ok(Some((_, msg))) = self.domain.try_recv_message() {
            if let Message::Topic { topic, data: _ } = &msg {
                for reader in &self.readers {
                    if let entity::Type::Reader(t) = &reader.0.kind {
                        if t == topic {
                            reader.1.lock().unwrap().message_cache.push(msg.clone());
                        }
                    }
                }
            } else {
                tracing::info!("received participant registry on unicast socket. ignoring.")
            }
        }

        Ok(())
    }

    pub fn process_all(&mut self) -> anyhow::Result<()> {
        self.process_writers()?;
        self.process_readers()?;
        Ok(())
    }
}
