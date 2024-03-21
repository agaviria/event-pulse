use std::collections::{HashMap, HashSet};

use crate::models::{decimal::Money, epoch::Epoch, signal::SignalTrigger, uid::GlobalId};
use crate::utils;

use chrono::{DateTime, Utc};
use log::{info, warn};
use structsy::derive::PersistentEmbedded;

// Separate type to represent time-triggered events
#[derive(Debug, Clone, PartialEq, PersistentEmbedded)]
pub struct Event {
    id: Vec<u8>,                       // UniqueId that is Url safe
    pub title: String,                 // Name of the event
    pub amount: Money,                 // Currency amount of the event
    pub epoch: Epoch,                  // Time scale for the event
    pub tags: Option<Vec<String>>,     // Classification for an event
    pub signal_trigger: SignalTrigger, // Military-Time trigger for the event
    pub start_datetime: DateTime<Utc>, // Start date for recorded event
    pub end_datetime: DateTime<Utc>,   // End date for recorded event
    created_at: DateTime<Utc>,         // Date created
}

impl Event {
    /// Constructor to create a new `Event`
    pub fn new(
        title: String,
        amount: Money,
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
    /// Create a public method in the Event struct to retrieve the id field.
    /// This method can be used by external code, including tests, to access the id.
    pub fn id(&self) -> &[u8] {
        &self.id
    }
}

pub struct EventManager {
    event_tags_map: HashMap<Vec<u8>, HashSet<String>>,
}

/// Manages events and their associated tags.
impl EventManager {
    /// Creates a new `EventManager`.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::event::EventManager;
    ///
    /// let event_manager = EventManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            event_tags_map: HashMap::new(),
        }
    }

    /// Tags an event with the provided tags.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to tag.
    /// * `tags` - The tags to associate with the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::event::{EventManager, Event};
    /// use event_pulse::models::{decimal::Money, uid::GlobalId, epoch::Epoch, SignalTrigger};
    /// use chrono::Utc;
    /// use std::{collections::HashSet, str::FromStr};
    ///
    /// let mut event_manager = EventManager::new();
    /// let tags = vec!["subscription".to_string(), "streaming-music".to_string()];
    /// let epoch = Epoch::from_str("1m12x").unwrap();
    /// let signal_trigger = SignalTrigger::from_str("M15:20:12::I60").expect("valid signal trigger");
    /// let event = Event::new(
    ///     "Spotify".to_string(),
    ///     Money::new(10, 99),
    ///     epoch,
    ///     Some(tags.clone()),
    ///     signal_trigger,
    ///     Utc::now(), // start-time
    ///     Utc::now(), // end-time
    /// );
    /// event_manager.tag_event(event.clone(), tags.clone());
    /// assert_eq!(event_manager.get_tags_for_event(&event.id()), Some(&tags.into_iter().collect::<HashSet<_>>()));
    /// ```
    pub fn tag_event(&mut self, event: Event, tags: Vec<String>) {
        let event_id = event.id.clone();
        for tag in tags.iter() {
            self.event_tags_map
                .entry(event_id.clone())
                .or_insert_with(HashSet::new)
                .insert(tag.clone());
        }
        info!("Event tagged with {:?}: {:?}", event_id, tags);
    }

    /// Retrieves the tags associated with the specified event ID.
    ///
    /// # Arguments
    ///
    /// * `event_id` - The ID of the event.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the set of tags associated
    /// with the event ID, or `None` if the event ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::event::{EventManager, Event};
    /// use event_pulse::models::{decimal::Money, uid::GlobalId, epoch::Epoch, SignalTrigger};
    /// use chrono::Utc;
    /// use std::{collections::HashSet, str::FromStr};
    ///
    /// let mut event_manager = EventManager::new();
    /// let tags = vec!["subscription".to_string(), "streaming-music".to_string()];
    /// let epoch = Epoch::from_str("1m12x").unwrap();
    /// let signal_trigger = SignalTrigger::from_str("M15:20:12::I60").expect("valid signal trigger");
    /// let event = Event::new(
    ///     "Spotify".to_string(),
    ///     Money::new(10, 99),
    ///     epoch,
    ///     Some(tags.clone()),
    ///     signal_trigger,
    ///     Utc::now(), // start-time
    ///     Utc::now(), // end-time
    /// );
    /// event_manager.tag_event(event.clone(), tags.clone()); // Convert HashSet<String> to Vec<String> for tag_event
    /// assert_eq!(event_manager.get_tags_for_event(&event.id()), Some(&tags.into_iter().collect::<HashSet<_>>()));
    /// ```
    pub fn get_tags_for_event(&self, event_id: &[u8]) -> Option<&HashSet<String>> {
        let tags = self.event_tags_map.get(event_id);
        if let Some(tags) = tags {
            info!(
                "Retrieved the following: Event:{:?}: Tag(s):{:?}",
                event_id, tags
            );
        } else {
            warn!("Event {:?} not found", event_id);
        }
        tags
    }

    /// Deletes a tag from all associated events.
    ///
    /// If the tag is associated with multiple events, a warning will be
    /// logged, and the tag will not be deleted.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to delete.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::event::EventManager;
    ///
    /// let mut event_manager = EventManager::new();
    /// event_manager.delete_tag("tag1");
    /// ```
    pub fn delete_tag(&mut self, tag: &str) {
        let mut more_than_one_association = false;
        let mut event_ids_to_remove: Vec<Vec<u8>> = Vec::new();

        for (event_id, tags) in self.event_tags_map.iter_mut() {
            if tags.contains(tag) {
                if tags.len() > 1 {
                    more_than_one_association = true;
                }
                tags.remove(tag);
                if tags.is_empty() {
                    event_ids_to_remove.push(event_id.clone());
                }
            }
        }

        if more_than_one_association {
            let affected_event_ids: Vec<_> = self
                .event_tags_map
                .iter()
                .filter(|(_, tags)| tags.contains(tag))
                .map(|(event_id, _)| event_id.clone())
                .collect();
            warn!("Tag '{}' was associated with more than one event. Removing the tag from the following event IDs: {:?}", tag, affected_event_ids);
        } else {
            info!("Tag '{}' deleted", tag);
        }

        for event_id in event_ids_to_remove {
            self.event_tags_map.remove(&event_id);
        }
    }

    /// Removes an event ID from its associated tag map.
    ///
    /// # Arguments
    ///
    /// * `event_id` - The ID of the event to remove.
    /// * `tag` - The tag from which to remove the event ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::event::EventManager;
    ///
    /// let mut event_manager = EventManager::new();
    /// let event_id = vec![1, 2, 3];
    /// event_manager.remove_event_from_tag(&event_id, "tag1");
    /// ```
    pub fn remove_event_from_tag(&mut self, event_id: &[u8], tag: &str) {
        if let Some(tags) = self.event_tags_map.get_mut(event_id) {
            tags.remove(tag);
            if tags.is_empty() {
                self.event_tags_map.remove(event_id);
            }
            info!("Event {:?} removed from tag '{}'", event_id, tag);
        } else {
            warn!("Event {:?} not found", event_id);
        }
    }
}
