use rust_decimal::Decimal as RustDecimal;
use std::fmt;
use structsy::derive::PersistentEmbedded;
use thiserror::Error;

/// Represents possible errors that can occur during `Money` operations.
#[derive(Error, Debug, PartialEq)]
pub enum MoneyError {
    /// Indicates the i64 value cannot be represented because it overflows.
    #[error("Value cannot be represented as i64")]
    ValueOverflow,
}

/// Represents a monetary amount consisting of a whole part and a fractional part.
///
/// # Example
///
/// ```
/// use event_pulse::models::decimal::Money;
///
/// let money = Money {
///     whole: 10,
///     fractional: 50,
/// };
/// ```
#[derive(Debug, PersistentEmbedded)]
pub struct Money {
    pub whole: i64,
    pub fractional: i64,
}

impl Money {
    /// Constructs a new `Money` instance with the specified whole and fractional parts.
    ///
    /// # Arguments
    ///
    /// * `whole` - The whole part of the monetary amount.
    /// * `fractional` - The fractional part of the monetary amount.
    ///
    /// # Returns
    ///
    /// A new `Money` instance representing the specified monetary amount.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::decimal::Money;
    ///
    /// let money = Money::new(10, 50);
    /// assert_eq!(money.whole, 10);
    /// assert_eq!(money.fractional, 50);
    /// ```
    pub fn new(whole: i64, fractional: i64) -> Self {
        Self { whole, fractional }
    }

    /// Constructs a `Money` instance from a `RustDecimal`.
    ///
    /// This method extracts the integral part and scale from the provided `RustDecimal`.
    /// If the scale is greater than 28, it normalizes it to 28 by adjusting the integral part.
    ///
    /// # Arguments
    ///
    /// * `decimal` - The `RustDecimal` from which to construct the `Money` instance.
    ///
    /// # Returns
    ///
    /// A new `Money` instance representing the monetary amount, or an error if the scale is too
    /// large or if the value cannot be represented as `i64`.
    ///
    /// # Errors
    ///
    /// This method may return an error of type `MoneyError` or panic under the following conditions:
    ///
    /// * If the scale of the provided `RustDecimal` is greater than 28, indicating that the scale
    /// is too large to represent a monetary amount accurately.
    ///
    /// * If the adjusted integral part cannot be represented as `i64`, indicating that the monetary
    /// amount is too large to fit within the range of a 64-bit signed integer.
    pub fn from_rust_decimal(decimal: RustDecimal) -> Result<Self, MoneyError> {
        // Extracting integral part and scale
        let (integral_part, scale) = (decimal.mantissa(), decimal.scale());

        // Adjusting the integral part based on the scale.
        // If the adjusted_integral part is too large for i64 and overflows it will panic
        // by rust_decimal.  We do not need to worry about error handling here in that case.
        let adjusted_integral = integral_part / 10_i128.pow(scale as u32);

        // Check if adjusted_integral fits within the range of i64
        if adjusted_integral > i64::MAX as i128 || adjusted_integral < i64::MIN as i128 {
            return Err(MoneyError::ValueOverflow);
        }

        // Convert the adjusted integral part to whole and fractional parts
        let whole = adjusted_integral as i64;
        let fractional = (integral_part % 10_i128.pow(scale as u32)) as i64;

        Ok(Self { whole, fractional })
    }
}

impl fmt::Display for Money {
    /// Formats the money value as a US dollar. Properly displays currency symbol
    /// and negative Money values.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the whole part with thousands separators
        let whole_str = format!("{:.*}", 0, self.whole.abs())
            .chars()
            .rev()
            .collect::<String>()
            .chars()
            .collect::<Vec<char>>()
            .chunks(3)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(",")
            .chars()
            .rev()
            .collect::<String>();

        // Format the fractional part with two digits after the decimal point
        let fractional_str = format!("{:02}", self.fractional.abs());

        // Write the formatted money value to the formatter
        write!(
            f,
            "${}{}.{}",
            if self.whole < 0 { "-" } else { "" },
            whole_str,
            fractional_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal as RustDecimal;

    #[test]
    fn test_from_rust_decimal() {
        // Test case 1: Decimal with scale <= 28
        let decimal1 = RustDecimal::new(31415926535897932, 2);
        let money_result = Money::from_rust_decimal(decimal1);
        assert!(money_result.is_ok());
        let money1 = money_result.unwrap();
        assert_eq!(money1.whole, 314159265358979);
        assert_eq!(money1.fractional, 32);

        // Test case 2: Decimal with scale > 28
        let decimal2 = RustDecimal::new(123456789012345678, 4);
        let money_result1 = Money::from_rust_decimal(decimal2);
        assert!(money_result1.is_ok());
        let money2 = money_result1.unwrap();
        assert_eq!(money2.whole, 12345678901234);
        assert_eq!(money2.fractional, 5678);

        // Test case 3: Decimal with scale = 0
        let decimal3 = RustDecimal::new(1234567891, 1);
        let money_result2 = Money::from_rust_decimal(decimal3);
        assert!(money_result2.is_ok());
        let money3 = money_result2.unwrap();
        assert_eq!(money3.whole, 123456789);
        assert_eq!(money3.fractional, 1);
    }

    #[test]
    fn test_display_positive_whole_and_fractional() {
        let money = Money {
            whole: 123456789,
            fractional: 50,
        };
        assert_eq!(money.to_string(), "$123,456,789.50");
    }

    #[test]
    fn test_display_positive_whole_and_no_fractional() {
        let money = Money {
            whole: 987654321,
            fractional: 0,
        };
        assert_eq!(money.to_string(), "$987,654,321.00");
    }

    #[test]
    fn test_display_positive_whole_and_single_digit_fractional() {
        let money = Money {
            whole: 123,
            fractional: 5,
        };
        assert_eq!(money.to_string(), "$123.05");
    }

    #[test]
    fn test_display_positive_whole_and_no_fractional_scale_zero() {
        let money = Money {
            whole: 987654321,
            fractional: 0,
        };
        assert_eq!(money.to_string(), "$987,654,321.00");
    }

    #[test]
    fn test_display_negative_whole_and_fractional() {
        let money = Money {
            whole: -123456789,
            fractional: -50,
        };
        assert_eq!(money.to_string(), "$-123,456,789.50");
    }

    #[test]
    fn test_display_negative_whole_and_no_fractional() {
        let money = Money {
            whole: -987654321,
            fractional: 3,
        };
        assert_eq!(money.to_string(), "$-987,654,321.03");
    }

    #[test]
    fn test_display_negative_whole_and_single_digit_fractional() {
        let money = Money {
            whole: -123,
            fractional: -5,
        };
        assert_eq!(money.to_string(), "$-123.05");
    }
}
