use crate::models::{event::Event, signal::SignalTrigger};
use chrono::{DateTime, Duration, Utc};
use structsy::derive::{Persistent, PersistentEmbedded};

/// Represents a recipient who can receive notifications.
#[derive(Debug, Clone, PartialEq, PersistentEmbedded)]
pub struct Recipient {
    /// Represents a recepient handle or email address, associated with `TransportMethod`
    recipient_id: String,
}

/// Represents the transport method of a notification.
#[derive(Debug, Clone, PartialEq, PersistentEmbedded)]
pub enum TransportMethod {
    /// notification method is via email (email_address).
    Email(Recipient),
    /// notifications method is via SMS (phone_number).
    Sms(Recipient),
    /// Sends notifications via push notification (subscriber).
    PushNotification(Recipient),
    /// Sends notifications via Slack (slack_handle)
    Slack(Recipient),
    /// Sends notifications via Discord (discord_handle)
    Discord(Recipient),
    /// Sends notifications via Telegram (telegram_handle)
    Telegram(Recipient),
}

/// Represents the calendar date frequency of notifications.
#[derive(Debug, Clone, PartialEq, PersistentEmbedded)]
pub enum SendFrequency {
    /// Sends notifications immediately when triggered.
    OnTrigger,
    /// Sends notifications one day prior to the event trigger.
    DayPrior,
    /// Sends notifications daily.
    Daily,
    /// Sends notifications weekly.
    Weekly,
    /// Sends notifications bi-weekly (every two weeks).
    BiWeekly,
    /// Sends notifications monthly.
    Monthly,
    /// Sends notifications quarterly.
    Quarterly,
}

/// Represents a notification to be sent for an event.
#[derive(Debug, Clone, PartialEq, Persistent)]
pub struct EventNotify {
    /// The ID of the event notification.
    id: Vec<u8>,
    /// The event for which the notification is scheduled.
    scheduled_event: Event,
    /// The method of notification delivery.
    delivery_method: TransportMethod,
    /// The frequency of notification delivery.
    delivery_frequency: SendFrequency,
    /// The recipients of the notification.
    recipients: Vec<Recipient>,
    /// The trigger for the notification (not the event trigger).
    notify_trigger: SignalTrigger,
    /// The start date of the notification.
    start_date: DateTime<Utc>,
    /// The date and time when the notification was created.
    created_at: DateTime<Utc>,
    /// The date and time when the notification was last updated.
    last_updated: DateTime<Utc>,
}

impl EventNotify {
    pub fn new(
        scheduled_event: Event,
        delivery_method: TransportMethod,
        delivery_frequency: SendFrequency,
        recipients: Vec<Recipient>,
        notify_trigger: SignalTrigger,
        start_date: DateTime<Utc>,
    ) -> Self {
        let id = crate::models::uid::GlobalId::new("NTFY").to_vec();
        let created_at = crate::utils::get_current_datetime_utc();
        let last_updated = created_at;

        EventNotify {
            id,
            scheduled_event,
            delivery_method,
            delivery_frequency,
            recipients,
            notify_trigger,
            start_date,
            created_at,
            last_updated,
        }
    }

    /// Adds a new recipient to the notification list.
    ///
    /// # Arguments
    ///
    /// * `recipient` - The recipient to add to the notification list.
    pub fn add_recipient(&mut self, recipient: Recipient) {
        self.recipients.push(recipient);
    }

    /// Removes a recipient from the notification list.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - The ID of the recipient to remove.
    pub fn remove_recipient(&mut self, recipient_id: &str) {
        self.recipients.retain(|r| r.recipient_id != recipient_id);
    }

    /// Updates the details of a recipient in the notification list.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - The ID of the recipient to update.
    /// * `new_recipient` - The updated details of the recipient.
    pub fn update_recipient(&mut self, recipient_id: &str, new_recipient: Recipient) {
        if let Some(index) = self
            .recipients
            .iter()
            .position(|r| r.recipient_id == recipient_id)
        {
            self.recipients[index] = new_recipient;
        }
    }

    /// Changes the notification frequency and adjusts the start date accordingly.
    ///
    /// # Arguments
    ///
    /// * `start_date` - The current start date of the notification.
    /// * `frequency` - The new notification frequency to set.
    ///
    /// # Remarks
    ///
    /// The start date is adjusted based on the new notification frequency. If
    /// the frequency is `OnTrigger`, the start date remains unchanged. For
    /// other frequencies, the start date is updated accordingly. The resulting
    /// start date is stored as a string due to compatibility requirements with
    /// the storage crate.
    pub fn edit_delivery_frequency(
        &mut self,
        start_date: DateTime<Utc>,
        deliver_frequency: SendFrequency,
    ) {
        use crate::models::time::from_duration_to_datetime;

        // Update notification frequency
        self.delivery_frequency = deliver_frequency;

        // Calculate new start date based on the frequency
        self.start_date = match self.delivery_frequency {
            SendFrequency::OnTrigger => start_date.to_utc(),
            SendFrequency::DayPrior => start_date,
            SendFrequency::Daily => {
                from_duration_to_datetime(start_date, Duration::try_days(1).unwrap())
            }
            SendFrequency::Weekly => {
                from_duration_to_datetime(start_date, Duration::try_weeks(1).unwrap())
            }
            SendFrequency::BiWeekly => {
                from_duration_to_datetime(start_date, Duration::try_weeks(2).unwrap())
            }
            SendFrequency::Monthly => {
                from_duration_to_datetime(start_date, Duration::try_days(30).unwrap())
            }
            SendFrequency::Quarterly => {
                from_duration_to_datetime(start_date, Duration::try_days(90).unwrap())
            }
        };
    }

    /// Sets a new event for notification.
    ///
    /// # Arguments
    ///
    /// * `event` - The new event details.
    pub fn set_event(&mut self, event: Event) {
        self.scheduled_event = event;
    }

    /// Triggers the notification manually.
    ///
    /// This method can be used to trigger a notification outside of the
    /// scheduled triggers.
    pub fn trigger_notification(&self) {
        // Logic to trigger notification goes here
    }

    /// Returns a list of all recipients for this notification.
    pub fn list_recipients(&self) -> Vec<&Recipient> {
        self.recipients.iter().collect()
    }

    /// Returns detailed information about the notification.
    ///
    /// This includes the event details, notification method, frequency,
    /// triggers remaining, and recipients.
    pub fn get_notification_details(&self) -> String {
        format!(
            "Event: {:?}, Method: {:?}, Frequency: {:?}, Recipients: {:?}",
            self.scheduled_event, self.delivery_method, self.delivery_frequency, self.recipients
        )
    }

    // Calculates the remaining number of notification triggers left, until the end of the event's life.
    // usage: get_trigger_count(Utc::now().naive_utc()),
    // pub fn get_trigger_count(&self, now: NaiveDateTime) -> i64 {
    //     let time_since_start = now.signed_duration_since(self.event.start_time);
    //     let time_since_last_trigger =
    //         time_since_start.num_seconds() % self.event.signal_trigger.interval;
    //     let remaining_time = self.event.signal_trigger.interval - time_since_last_trigger;
    //     let notify_trigger_count = remaining_time / self.event.signal_trigger.interval;

    //     notify_trigger_count
    // }
}

#[test]
fn test_edit_delivery_frequency() {
    use crate::models::decimal::Money;
    // Create initial data
    let start_date = Utc::now();
    // Create amount of type models::decimal::Money
    let amount: Money = Money::new(100, 50);
    // -- assign signal_trigger(s)
    let raw_trigger = "M15:20:12::I60";
    let parsed_signal = SignalTrigger::from_str(raw_trigger);
    // -- assign epoch
    let raw_epoch = "15d3x";
    let parsed_epoch = <crate::models::Epoch as std::str::FromStr>::from_str(raw_epoch).unwrap();
    // -- create DateTime<Utc>
    // Get the current local time
    let local_time = chrono::Local::now();
    // Convert the local time to UTC
    let utc_time = local_time.with_timezone(&Utc);
    // tags for event entry
    let tags: Option<Vec<String>> = Some(vec!["boat".to_owned(), "rental".to_owned()]);

    let recipient = Recipient {
        recipient_id: "test@example.com".to_string(),
    };

    let delivery_method = TransportMethod::Email(recipient.clone());
    let delivery_frequency = SendFrequency::Weekly;
    let notify_trigger = SignalTrigger::new(
        crate::models::time::MilitaryTime {
            hour: 10,
            minute: 0,
            seconds: 0,
        },
        crate::SECS_IN_WEEK,
    );

    let scheduled_event = Event::new(
        "fishing boat rental".into(),
        amount,
        parsed_epoch.clone(),
        tags,
        parsed_signal.unwrap(),
        utc_time.clone(),
        crate::models::time::from_duration_to_datetime(
            utc_time.clone(),
            crate::models::epoch::Epoch::to_duration(&parsed_epoch),
        ),
    );

    let mut event_notify = crate::models::notify::EventNotify::new(
        scheduled_event,
        delivery_method,
        delivery_frequency,
        vec![recipient],
        notify_trigger,
        utc_time,
    );

    // Test with different frequencies
    event_notify.edit_delivery_frequency(start_date, SendFrequency::DayPrior);
    assert_eq!(event_notify.start_date, start_date);

    event_notify.edit_delivery_frequency(start_date, SendFrequency::Daily);
    assert_eq!(
        event_notify.start_date,
        start_date + Duration::try_days(1).unwrap()
    );

    event_notify.edit_delivery_frequency(start_date, SendFrequency::Weekly);
    assert_eq!(
        event_notify.start_date,
        start_date + Duration::try_weeks(1).unwrap()
    );
}
