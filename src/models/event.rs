use crate::models::{epoch::Epoch, signal::SignalTrigger, uid::GlobalId};
use crate::utils;

use chrono::{DateTime, Utc};
use structsy::derive::PersistentEmbedded;

// Separate type to represent time-triggered events
#[derive(Debug, Clone, PartialEq, PersistentEmbedded)]
pub struct Event {
    id: Vec<u8>,                   // UniqueId that is Url safe
    title: String,                 // Name of the event
    amount: f64,                   // Currency amount of the event
    epoch: Epoch,                  // Time scale for the event
    tags: Option<Vec<String>>,     // Classification for an event
    signal_trigger: SignalTrigger, // Military-Time trigger for the event
    start_datetime: DateTime<Utc>, // Start date for recorded event
    end_datetime: DateTime<Utc>,   // End date for recorded event
    created_at: DateTime<Utc>,     // Date created
}

impl Event {
    /// Constructor to create a new `Event`
    pub fn new(
        title: String,
        amount: f64,
        epoch: Epoch,
        tags: Option<Vec<String>>,
        signal_trigger: SignalTrigger,
        start_datetime: DateTime<Utc>,
        end_datetime: DateTime<Utc>,
    ) -> Self {
        let id = GlobalId::new("EVNT").to_vec();
        let created_at = utils::get_current_datetime_utc();
        Self {
            id,
            title,
            amount,
            epoch,
            tags,
            signal_trigger,
            start_datetime,
            end_datetime,
            created_at,
        }
    }
}
