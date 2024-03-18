use chrono::NaiveTime;
use structsy::derive::PersistentEmbedded;

use crate::error::AppError;

/// 24-Hour clock also known as military time
#[derive(Debug, Clone, PartialEq, PersistentEmbedded)]
pub struct MilitaryTime {
    /// Represents 24-Hour time format (hour, minute, seconds)
    pub hour: u32,
    pub minute: u32,
    pub seconds: u32,
}

impl MilitaryTime {
    /// Constructs a new `MilitaryTime`.
    pub fn new(hour: u32, minute: u32, seconds: u32) -> Self {
        MilitaryTime {
            hour,
            minute,
            seconds,
        }
    }

    /// Parses a string representation of military time into a `MilitaryTime` instance.
    ///
    /// The input string should be formatted as "HH:MM:SS", where:
    /// - `HH` represents the hour component in 24-hour format.
    /// - `MM` represents the minute component.
    /// - `SS` represents the second component.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice containing the formatted military time.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `MilitaryTime` instance if parsing is successful,
    /// or an `AppError` if parsing fails due to invalid format or other errors.
    ///
    /// # Example
    ///
    /// ```
    /// use event_pulse::models::MilitaryTime;
    ///
    /// let military_time = MilitaryTime::from_str("16:30:25");
    /// assert!(military_time.is_ok());
    /// ```
    pub fn from_str(input: &str) -> Result<MilitaryTime, AppError> {
        let parts: Vec<&str> = input.trim().split(':').collect();
        if parts.len() != 3 {
            tracing::error!("Invalid military time format");
            return Err(AppError::InvalidInputString(
                "Invalid military time format".to_string(),
            ));
        }

        let hour = parts[0].parse().map_err(|_| {
            tracing::error!("Failed to parse military time hour");
            AppError::ParseError("Failed to parse military time hour".to_string())
        })?;

        let minute = parts[1].parse().map_err(|_| {
            tracing::error!("Failed to parse military time minute");
            AppError::ParseError("Failed to parse military time minute".to_string())
        })?;

        let seconds = parts[2].parse().map_err(|_| {
            tracing::error!("Failed to parse military time seconds");
            AppError::ParseError("Failed to parse military time seconds".to_string())
        })?;

        Ok(MilitaryTime {
            hour,
            minute,
            seconds,
        })
    }

    /// Converts `MilitaryTime` to `chrono::NaiveTime`
    /// Structsy v0.5 does not currently handle `chrono` support,
    /// therefore we use `MilitaryTime` of type `u32`
    /// as our concrete type and use `to_naive_time()` to convert into `chrono::NaiveTime`.
    pub fn to_naive_time(&self) -> NaiveTime {
        NaiveTime::from_hms_opt(self.hour, self.minute, self.seconds)
            .expect("expected conversion to chrono::NaiveTime")
    }

    /// Converts chrono::NaiveTime to MilitaryTime.
    ///
    /// This method takes a chrono::NaiveTime and extracts the hour, minute, and second
    /// components to create a MilitaryTime instance.
    ///
    /// # Arguments
    ///
    /// * time - A chrono::NaiveTime instance.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::NaiveTime;
    /// use event_pulse::models::MilitaryTime;
    ///
    /// let naive_time = NaiveTime::from_hms_opt(12, 30, 0).expect("chrono::NaiveTime");
    /// let military_time = MilitaryTime::from_naive_time(naive_time);
    ///
    /// assert_eq!(military_time.hour, 12);
    /// assert_eq!(military_time.minute, 30);
    /// assert_eq!(military_time.seconds, 0);
    /// ```
    pub fn from_naive_time(time: NaiveTime) -> Self {
        use chrono::Timelike;

        MilitaryTime {
            hour: time.hour(),
            minute: time.minute(),
            seconds: time.second(),
        }
    }
}

/// Adds a chrono::Duration to a chrono::DateTime<Utc> and returns the result.
///
/// # Arguments
///
/// * `datetime`: A chrono::DateTime<Utc> to which the duration will be added.
/// * `duration`: A chrono::Duration to add to the datetime.
///
/// # Returns
///
/// * `DateTime<Utc>`: The resulting datetime after adding the duration.
///
/// # Example
///
/// ```
/// use chrono::{DateTime, Utc, Duration};
/// use event_pulse::models::time::from_duration_to_datetime;
///
/// let datetime = Utc::now();
/// let duration = Duration::try_days(1);
///
/// let result = from_duration_to_datetime(datetime, duration.expect("chrono::Duration"));
///
/// assert_eq!(result, datetime + duration.expect("chrono::Duration"));
/// ```
pub fn from_duration_to_datetime(
    datetime: chrono::DateTime<chrono::Utc>,
    duration: chrono::Duration,
) -> chrono::DateTime<chrono::Utc> {
    datetime + duration
}
