use std::str::FromStr;

use chrono::{NaiveDateTime, NaiveTime, Timelike};
use once_cell::sync::Lazy;
use regex::Regex;
use structsy::derive::PersistentEmbedded;

use crate::error::AppError;

static RE_EPOCH: Lazy<Regex> = Lazy::new(|| init_regex_epoch());

fn init_regex_epoch() -> Regex {
    Regex::new(r"(([1-9]{1}[0-9]*)([dwmy]))(([1-9]{1}[0-9]*)x)?")
        .expect("failed to initialize epoch regex")
}

/// Represents the calendar data set for an epoch duration, associated with
/// amount: i64 and coefficient: i64.
#[derive(Debug, Copy, Clone, PartialEq, PersistentEmbedded)]
pub struct CalendarData {
    pub amount: i64,
    pub coefficient: i64,
}

impl CalendarData {
    pub fn new(amount: i64, coefficient: i64) -> Self {
        Self {
            amount,
            coefficient,
        }
    }
}

/// A time range represented by various units with duration and coefficient.
///
/// An `Epoch` can represent time duration in units such as year(s), month(s),
/// week(s), days, and single_day.
#[derive(Debug, Copy, Clone, PartialEq, PersistentEmbedded)]
pub enum Epoch {
    /// Represents a single day.
    SingleDay,
    /// Represents a duration in years with an associated tuple, CalendarData { amount, coefficient }
    Year(CalendarData),
    /// Represents a duration in months with an associated tuple, CalendarData { amount, coefficient }
    Month(CalendarData),
    /// Represents a duration in weeks with an associated tuple, CalendarData { amount, coefficient }
    Week(CalendarData),
    /// Represents a duration in days with an associated tuple, CalendarData { amount, coefficient }
    Day(CalendarData),
}

impl Epoch {
    /// Creates a new `Epoch` with the specified unit, and tuple:
    /// CalendarData {amount: i64, coefficient: i64}
    ///
    /// # Arguments
    ///
    /// * `unit` - A string slice representing the unit of time ('y' for year, 'm' for month,
    ///            'w' for week, 'd' for day).
    /// * CalendarData - tuple with `amount`, `coefficient`
    /// * `amount` - The amount of time for the given unit.
    /// * `coefficient` - The coefficient or frequency associated with the duration.
    ///
    /// # Returns
    ///
    /// A new `Epoch` instance representing the specified time range.
    ///
    /// # Example
    ///
    /// ```
    /// use event_pulse::models::{CalendarData, Epoch};
    ///
    /// let year2_1x = CalendarData::new(2, 1);
    /// let epoch = Epoch::new("y", year2_1x);
    /// assert_eq!(epoch, Epoch::Year( CalendarData { amount: 2, coefficient: 1 }));
    /// ```
    pub fn new(unit: &str, calendar_data: CalendarData) -> Self {
        match unit {
            "y" => Epoch::Year(calendar_data),
            "m" => Epoch::Month(calendar_data),
            "w" => Epoch::Week(calendar_data),
            "d" => Epoch::Day(calendar_data),
            _ => Epoch::SingleDay,
        }
    }

    /// Returns the frequency coefficient associated with the `Epoch`.
    ///
    /// For `SingleDay`, returns 1 as it represents a single day. For other variants
    /// (`Year`, `Month`, `Week`, `Day`), returns the coefficient value from the
    /// associated `CalendarData`.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::{CalendarData, Epoch};
    ///
    /// let year_data = CalendarData::new(1, 3);
    /// let year_epoch = Epoch::Year(year_data);
    /// assert_eq!(year_epoch.get_frequency(), 3);
    ///
    /// let single_day_epoch = Epoch::SingleDay;
    /// assert_eq!(single_day_epoch.get_frequency(), 1);
    /// ```
    pub fn get_frequency(&self) -> i64 {
        match self {
            Self::SingleDay => 1,
            Self::Year(calendar_data)
            | Self::Month(calendar_data)
            | Self::Week(calendar_data)
            | Self::Day(calendar_data) => calendar_data.coefficient,
        }
    }

    /// Returns the number of days from a given date, typically a `start_date`
    ///
    /// This method calculates the number of days based on the epoch type and the
    /// provided starting date (`since`). It considers months and leap years to
    /// compute the exact number of days.
    ///
    /// # Arguments
    ///
    /// * `since` - The starting date from which to calculate the number of days.
    ///
    /// # Returns
    ///
    /// The number of days from the given date based on the epoch type.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::{Epoch, CalendarData};
    /// use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
    ///
    /// let epoch = Epoch::Month(CalendarData::new(1, 1));
    /// let since = NaiveDateTime::new(NaiveDate::from_ymd_opt(2024, 2, 1).expect("NaiveDate"), NaiveTime::from_hms_opt(0, 0, 0).expect("NaiveTime"));
    /// let days_elapsed = epoch.calculate_days_since(since);
    /// assert_eq!(days_elapsed, 29); // tests leap month duration
    /// ```
    pub fn calculate_days_since(&self, since: NaiveDateTime) -> i64 {
        use chrono::{Datelike, NaiveDate};

        let since_date = since.date();
        match self {
            Self::Month(cd) => {
                let mut current_year = since_date.year();
                let mut current_month = since_date.month() as i32;
                for _ in 0..cd.coefficient {
                    current_month += cd.amount as i32;
                    if current_month > 12 {
                        current_year += 1;
                        current_month -= 12;
                    }
                }
                let end_datetime = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(current_year, current_month as u32, since_date.day())
                        .expect("failed to create NaiveDate from NaiveDateTime"),
                    NaiveTime::from_hms_opt(
                        since.time().hour(),
                        since.time().minute(),
                        since.time().second(),
                    )
                    .expect("failed to create NaiveTime"),
                );
                end_datetime.signed_duration_since(since).num_days()
            }
            Self::Year(cd) => {
                let end_year = since_date.year() + (cd.coefficient * cd.amount) as i32;
                let end_datetime = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(end_year, since_date.month(), since_date.day())
                        .expect("failed to create NaiveDate from NaiveDateTime"),
                    NaiveTime::from_hms_opt(
                        since.time().hour(),
                        since.time().minute(),
                        since.time().second(),
                    )
                    .expect("failed to create NaiveTime"),
                );
                end_datetime.signed_duration_since(since).num_days()
            }
            Self::Week(cd) => cd.amount * cd.coefficient * crate::DAYS_IN_WEEK as i64,
            Self::Day(cd) => cd.amount * cd.coefficient,
            Self::SingleDay => 1,
        }
    }

    /// Converts the Epoch variant into a chrono Duration representing the
    /// duration in days.
    ///
    /// # Returns
    ///
    /// A chrono Duration representing the duration in days.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::{Epoch, CalendarData};
    /// use chrono::Duration;
    ///
    /// // Create an Epoch representing a year duration
    /// let year_duration = Epoch::Year(CalendarData::new(1, 1)).to_duration();
    ///
    /// // Verify that the duration corresponds to the expected number of days in a year
    /// assert_eq!(year_duration, Duration::try_days(365).expect("invalid number of days"));
    /// ```
    pub fn to_duration(&self) -> chrono::Duration {
        use chrono::Duration;

        match self {
            Self::SingleDay => Duration::try_days(1).expect("1 day"),
            Self::Year(calendar_data) => Duration::try_days(
                (calendar_data.amount * calendar_data.coefficient * crate::DAYS_IN_YEAR) as i64,
            )
            .expect("Invalid number of days"),
            Self::Month(calendar_data) => Duration::try_days(
                (calendar_data.amount * calendar_data.coefficient * crate::DAYS_IN_MONTH) as i64,
            )
            .expect("Invalid number of days"),
            Self::Week(calendar_data) => Duration::try_days(
                (calendar_data.amount * calendar_data.coefficient * crate::DAYS_IN_WEEK) as i64,
            )
            .expect("Invalid number of days"),
            Self::Day(calendar_data) => {
                Duration::try_days((calendar_data.amount * calendar_data.coefficient) as i64)
                    .expect("Invalid number of days")
            }
        }
    }
}

fn parse_epoch(text: &str) -> (&str, i64, i64) {
    match RE_EPOCH.captures(text) {
        Some(c) => (
            c.get(3).map_or("d", |unit| unit.as_str()),
            c.get(2).map_or(1, |a| a.as_str().parse::<i64>().unwrap()),
            c.get(5).map_or(1, |r| r.as_str().parse::<i64>().unwrap()),
        ),
        None => ("d", 1, 1),
    }
}

impl FromStr for Epoch {
    type Err = AppError;

    /// Parses a string slice into an `Epoch` instance.
    ///
    /// This function parses the input string `s` to determine the time unit,
    /// amount, and coefficient of an epoch. It then constructs and returns
    /// the corresponding `Epoch` enum variant.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice containing the epoch information to parse.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Epoch` instance if successful,
    /// or an `AppError` if parsing fails due to invalid input.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::{CalendarData, Epoch};
    /// use std::str::FromStr;
    ///
    /// let epoch = Epoch::from_str("1d").unwrap();
    /// assert_eq!(epoch, Epoch::SingleDay);
    ///
    /// let epoch = Epoch::from_str("3m4x").unwrap();
    /// assert_eq!(epoch, Epoch::Month(CalendarData { amount: 3, coefficient: 4 }));
    /// ```
    fn from_str(s: &str) -> Result<Epoch, AppError> {
        let (unit, amount, coefficient) = parse_epoch(s);
        match unit {
            "y" => Ok(Epoch::Year(CalendarData {
                amount,
                coefficient,
            })),
            "m" => Ok(Epoch::Month(CalendarData {
                amount,
                coefficient,
            })),
            "w" => Ok(Epoch::Week(CalendarData {
                amount,
                coefficient,
            })),
            "d" => {
                if s == "1d1x" || s == "1d" {
                    Ok(Epoch::SingleDay)
                } else {
                    Ok(Epoch::Day(CalendarData {
                        amount,
                        coefficient,
                    }))
                }
            }
            _ => Err(AppError::InvalidInputString(
                "Encounterd invalid epoch token unit from string".to_string(),
            )),
        }
    }
}

impl std::fmt::Display for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Year(cd) => write!(f, "{}y{}x", cd.amount, cd.coefficient),
            Self::Month(cd) => write!(f, "{}m{}x", cd.amount, cd.coefficient),
            Self::Week(cd) => write!(f, "{}w{}x", cd.amount, cd.coefficient),
            Self::Day(cd) => write!(f, "{}d{}x", cd.amount, cd.coefficient),
            Self::SingleDay => write!(f, "1d1x"),
        }
    }
}
