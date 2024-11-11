use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::participant::RemoteParticipant;

#[derive(Debug, Clone, Deserialize, Serialize, From)]
pub enum Message {
    ParticipantRegister(RemoteParticipant),
    Topic { topic: String, data: String },
}
