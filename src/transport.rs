use serde::{Deserialize, Serialize};

use crate::domain::{ExchangePanelSnapshot, VenueId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerRequestEnvelope {
    LoadDashboard,
    SelectVenue { venue: VenueId },
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerResponseEnvelope {
    pub snapshot: ExchangePanelSnapshot,
}
