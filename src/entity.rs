use serde::{Deserialize, Serialize};

type Id = u32;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Type {
    Writer(String),
    Reader(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Entity {
    pub id: Id,
    pub kind: Type,
}
