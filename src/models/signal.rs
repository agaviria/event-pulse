use crate::error::AppError;
use crate::models::time::MilitaryTime;
use structsy::derive::PersistentEmbedded;

/// Defines a designated point-in-time (MilitaryTime) and the sleep duration
/// measured in seconds.  SignaLTrigger stores a time-scale, useful to trigger
/// an event or notification alert.
#[derive(Debug, Clone, PartialEq, PersistentEmbedded)]
pub struct SignalTrigger {
    // military time formated, used as NaiveTime to execute a signal trigger.
    pub time: MilitaryTime,
    // interval refers to the time-span in-between triggers, measured in seconds.
    pub interval_seconds: i64,
}

impl SignalTrigger {
    /// Creates a new SignalTrigger with the specified `time` and `interval` types.
    pub fn new(time: MilitaryTime, interval_seconds: i64) -> Self {
        Self {
            time,
            interval_seconds,
        }
    }

    /// Parses a string representation of a signal trigger into a `SignalTrigger` instance.
    ///
    /// The input string should be formatted as "MHH:MM:SS::Ii64", where:
    /// - `M` is a `char` delimiter indicating the start of the time part.
    /// - `HH` represents the hour component in 24-hour format.
    /// - `MM` represents the minute component.
    /// - `SS` represents the second component.
    /// - `::` is a delimiter to separate end of time string and start of interval string.
    /// - `I` is a `char` delimeter to indicate start of interval.
    /// - `i64` represents the interval between signals, captured as seconds.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice containing the formatted signal trigger.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `SignalTrigger` instance if parsing is successful,
    /// or an `AppError` if parsing fails due to invalid format or other errors.
    ///
    /// # Example
    ///
    /// ```
    /// use event_pulse::models::{MilitaryTime, SignalTrigger};
    ///
    /// let expected_trigger = SignalTrigger {
    ///     time: MilitaryTime::new(16, 30, 25),
    ///     interval_seconds: 86400,
    /// };
    /// let signal_trigger = SignalTrigger::from_str("M16:30:25::I86400");
    /// assert_eq!(signal_trigger.unwrap(), expected_trigger);
    /// ```
    pub fn from_str(input: &str) -> Result<SignalTrigger, AppError> {
        // Splitting input by '::I' to separate time and interval parts
        let parts: Vec<&str> = input.trim().split("::I").collect();
        if parts.len() != 2 {
            tracing::error!("Invalid signal trigger format");
            return Err(AppError::InvalidInputString(
                "Invalid signal trigger format".to_string(),
            ));
        }

        // Parsing time part into MilitaryTime
        let time_str = parts[0].trim_start_matches('M');
        let time = match MilitaryTime::from_str(time_str) {
            Ok(time) => time,
            Err(err) => {
                tracing::error!("Failed to parse signal trigger time: {}", err);
                return Err(AppError::ParseError(format!(
                    "Failed to parse signal trigger time: {}",
                    err
                )));
            }
        };

        // Parsing interval part into i64
        let interval_seconds = parts[1].parse().map_err(|_| {
            tracing::error!("Failed to parse signal trigger interval");
            AppError::ParseError("Failed to parse signal trigger interval".to_string())
        })?;

        log::debug!(
            "Signal trigger parsed successfully: {:?}",
            SignalTrigger::new(time.clone(), interval_seconds)
        );

        Ok(SignalTrigger::new(time, interval_seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_valid_input() {
        // Valid input: "M16:30:25::I86400"
        let input = "M16:30:25::I86400";
        let expected_time = MilitaryTime::new(16, 30, 25);
        let expected_trigger = SignalTrigger::new(expected_time, 86400);

        assert_eq!(SignalTrigger::from_str(input), Ok(expected_trigger));
    }

    #[test]
    fn test_from_str_invalid_format() {
        // Invalid input: missing time component
        let input = "::I86400";
        let result = SignalTrigger::from_str(input);
        assert!(result.is_err());

        // Invalid input: missing interval component
        let input = "M16:30:25::";
        let result = SignalTrigger::from_str(input);
        assert!(result.is_err());

        // Invalid input: missing both time and interval components
        let input = "::";
        let result = SignalTrigger::from_str(input);
        assert!(result.is_err());

        // Invalid input: missing delimiter and interval component
        let input = "16:30:25::";
        let result = SignalTrigger::from_str(input);
        assert!(result.is_err());

        // Invalid input: missing time component
        let input = "M::I86400";
        let result = SignalTrigger::from_str(input);
        assert!(result.is_err());

        // Invalid input: missing interval value
        let input = "M16:30:25::I";
        let result = SignalTrigger::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_invalid_interval() {
        // Invalid input: invalid interval value
        let input = "M16:30:25::Iinvalid";
        let result = SignalTrigger::from_str(input);
        assert!(result.is_err());
    }
}
