use serde::{Deserialize, Serialize};
use derive_more::From;

use crate::entity::Entity;

#[derive(Debug, Clone, Deserialize, Serialize, From)]
pub enum Message {
    EntityDiscovery(EntityDiscovery)
}

#[derive(Debug, Clone, Deserialize, Serialize, From)]
pub struct EntityDiscovery(pub Entity);
