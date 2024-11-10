use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use socket2::SockAddr;

use crate::domain::DomainConnection;
use crate::entity::{Entity, ReaderState, SharedReaderState, SharedWriterState, WriterState};
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
}

impl RTPSParticipant {
    pub fn new(domain: DomainConnection) -> Self {
        Self {
            domain,
            remote_participants: HashMap::new(),
            writers: HashMap::new(),
            readers: HashMap::new(),
        }
    }

    pub fn new_writer<T: ToString>(&mut self, topic: T) -> SharedWriterState {
        let state = WriterState::new();
        self.writers
            .insert(Entity::new_reader(topic), state.clone());
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
        if let Ok(Some((addr, msg))) = self.domain.try_recv_message_discovery() {
            if let Message::ParticipantRegister(pr) = msg {
                self.remote_participants.insert(addr.into(), pr);
            }
        }

        Ok(())
    }
}
