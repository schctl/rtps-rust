use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::message::Message;

type Id = u32;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum Type {
    Writer(String),
    Reader(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct Entity {
    pub kind: Type,
}

impl Entity {
    pub fn new_reader<T: ToString>(topic: T) -> Self {
        Self {
            kind: Type::Reader(topic.to_string()),
        }
    }

    pub fn new_writer<T: ToString>(topic: T) -> Self {
        Self {
            kind: Type::Writer(topic.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriterState {
    message_cache: Vec<Message>,
}

impl WriterState {
    pub(crate) fn new() -> SharedWriterState {
        Arc::new(Mutex::new(WriterState {
            message_cache: Vec::new(),
        }))
    }

    pub fn write(&mut self, message: &Message) {
        self.message_cache.push(message.clone());
    }

    pub fn clear(&mut self) {
        self.message_cache.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ReaderState {
    message_cache: Vec<Message>,
}

impl ReaderState {
    pub(crate) fn new() -> SharedReaderState {
        Arc::new(Mutex::new(ReaderState {
            message_cache: Vec::new(),
        }))
    }

    pub fn pop(&mut self) -> Vec<Message> {
        let cloned = self.message_cache.clone();
        self.message_cache.clear();
        cloned
    }
}

pub type SharedWriterState = Arc<Mutex<WriterState>>;
pub type SharedReaderState = Arc<Mutex<ReaderState>>;
