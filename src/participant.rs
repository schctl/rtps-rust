use std::collections::HashMap;

use crate::domain::RTPSDomain;

pub struct RTPSParticipant {
    domain: RTPSDomain,
    participant_cache: HashMap<>    
}
