use once_cell::sync::Lazy;
use regex::Regex;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

use pxid::{Error as PxidError, Factory}; // todo implement  code commented out below:

/// Represents a prefix, component for xid that has a boundary constraint of 4 bytes.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Prefix(String);

impl Prefix {
    /// Creates a new `Prefix` instance.
    ///
    /// The provided prefix must not exceed 4 bytes in length. Prefix is a component part used to create a xid.
    /// ```ignore
    /// V V V V W W W W X X X Y Y Z Z Z
    /// └─────┘ └─────────────────────┘
    ///    |              |
    /// Prefix            |
    ///                   |
    ///                   |
    ///                  XID
    /// ```
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix value.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Prefix` instance if successful, or a `PrefixError` if the prefix exceeds 4 bytes.
    pub fn new(prefix: &str) -> Result<Self, UniqueIdentifierError> {
        if prefix.len() > 4 {
            Err(UniqueIdentifierError::InvalidPrefix)
        } else {
            Ok(Prefix(prefix.to_string()))
        }
    }
}

impl TryFrom<String> for Prefix {
    type Error = UniqueIdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Prefix::new(&value)
    }
}

impl FromStr for Prefix {
    type Err = UniqueIdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Prefix::new(s)
    }
}

/// Regular expression pattern for validating a unique identifier (UID).
static RE_UID: Lazy<Regex> = Lazy::new(|| init_regex_valid_uid());

/// Initializes and returns a regular expression for validating a unique identifier (UID).
///
/// This function compiles a regular expression pattern used to validate unique identifiers.
///
/// # Returns
///
/// A `Regex` instance representing the compiled regular expression pattern.
fn init_regex_valid_uid() -> Regex {
    Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").expect("Failed to compile unique identifier regex")
}

#[derive(Error, Debug)]
pub enum UniqueIdentifierError {
    #[error("Invalid unique identifier: {0}")]
    InvalidUniqueIdentifier(String),
    #[error("Failed to create Factory for xid: {0}")]
    FactoryCreationError(#[from] PxidError),
    #[error("Failed to create Uid: Invalid prefix length for xid")]
    InvalidPrefix,
}

/// Represents a valid identifier.
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub(crate) struct UniqueIdentifier {
    uid: &'static str,
}

impl UniqueIdentifier {
    // Constructor method
    pub fn new(uid: &'static str) -> Self {
        UniqueIdentifier { uid }
    }

    /// Creates a new identifier with a prefix using the provided factory.
    ///
    /// # Arguments
    ///
    /// * `factory` - The factory used to generate the identifier.
    /// * `prefix` - The prefix to prepend to the generated identifier.
    ///
    /// # Errors
    ///
    /// Returns a `UniqueIdentifierError` if the identifier creation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use event_pulse::models::{UniqueIdentifier, uid::UniqueIdentifierError, uid::Prefix};
    /// use pxid::Factory;
    ///
    /// # fn main() -> Result<(), UniqueIdentifierError> {
    /// let prefix = Prefix::new("test")?;
    /// let uid = UniqueIdentifier::new_with_prefix(prefix)?;
    /// assert!(uid.to_string().starts_with("test"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_prefix(prefix: Prefix) -> Result<Self, UniqueIdentifierError> {
        let factory = Factory::new()?;
        let uid_with_prefix = Factory::new_id(&factory, &prefix.0)?;
        let static_ref: &'static str = Box::leak(uid_with_prefix.to_string().into_boxed_str());
        Ok(Self { uid: static_ref })
    }

    /// Determines whether a string value is a valid unique identifier (UID).
    ///
    /// # Arguments
    ///
    /// * `s` - The string value to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the string value is a valid unique identifier (UID), `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use once_cell::sync::Lazy;
    /// use regex::Regex;
    ///
    /// static RE_UID: Lazy<Regex> = Lazy::new(|| init_regex_valid_uid());
    ///
    /// fn init_regex_valid_uid() -> Regex {
    ///     Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").expect("Failed to compile unique identifier regex")
    /// }
    ///
    /// assert_eq!(event_pulse::models::UniqueIdentifier::is_valid_identifier("some_uid_str"), true);
    /// assert_eq!(event_pulse::models::UniqueIdentifier::is_valid_identifier("123"), false);
    /// ```
    pub fn is_valid_identifier(s: &str) -> bool {
        RE_UID.is_match(s)
    }

    /// The inner string value of the identifier.
    ///
    /// # Returns
    ///
    /// A reference to the inner string value of the identifier.
    pub const fn into_inner(&self) -> &str {
        self.uid
    }
}

/// The `Display` trait implementation enables formatting an `Identifier` for display purposes.
/// It allows converting an `Identifier` instance to a string representation.
///
/// # Example
///
/// ```
/// use std::fmt::Display;
/// use event_pulse::models::UniqueIdentifier;
/// use event_pulse::models::uid::Prefix;
///
/// let pre = Prefix::new("uid").unwrap_or_default();
/// let uid = UniqueIdentifier::new_with_prefix(pre).unwrap();
/// println!("Uid: {}", uid);
/// ```
///
impl Display for UniqueIdentifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.uid)
    }
}

impl Deref for UniqueIdentifier {
    type Target = &'static str;

    fn deref(&self) -> &Self::Target {
        &self.uid
    }
}

impl TryFrom<Prefix> for UniqueIdentifier {
    type Error = UniqueIdentifierError;

    fn try_from(value: Prefix) -> Result<Self, Self::Error> {
        UniqueIdentifier::new_with_prefix(value)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum UniqueIdentifierType {
    String,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum UniqueIdentifierValue {
    String(String),
}

impl std::fmt::Display for UniqueIdentifierValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::String(value) => write!(f, "{}", value),
        }
    }
}

impl IntoUniqueIdentifierValue for &str {
    const TYPE: UniqueIdentifierType = UniqueIdentifierType::String;
    fn into_unique_identifier_value(self) -> UniqueIdentifierValue {
        UniqueIdentifierValue::String(self.to_string())
    }
}

/// Represents a value that can be used as an identifier value.
///
/// The `IntoUniqueIdentifierValue` trait allows converting values into `IdentifierValue` instances,
/// specifying the type of the value.
///
/// Implementations of this trait should provide the associated constant `TYPE` with the specific
/// `IdentifierType` variant corresponding to the type being converted.
pub trait IntoUniqueIdentifierValue {
    /// the type of the value
    const TYPE: UniqueIdentifierType;
    /// Converts the value into the corresponding `IdentifierValue` variant.
    ///
    /// # Returns
    ///
    /// An `IdentifierValue` variant containing the converted value.
    fn into_unique_identifier_value(self) -> UniqueIdentifierValue;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_prefix() {
        let prefix1 = match crate::models::uid::Prefix::new("acct") {
            Ok(prefix) => prefix,
            Err(err) => {
                // Handle the error here
                panic!("Error creating prefix1: {:?}", err);
            }
        };
        let prefix2 = match crate::models::uid::Prefix::new("") {
            Ok(prefix) => prefix,
            Err(err) => {
                // Handle the error here
                panic!("Error creating prefix2: {:?}", err);
            }
        };
        let prefix3 = crate::models::uid::Prefix::new("invalid prefix").unwrap_or_default();

        // Test with a valid prefix
        let result = UniqueIdentifier::new_with_prefix(prefix1);
        assert!(result.is_ok());
        let pxid = result.unwrap();
        assert!(pxid.uid.starts_with("acct_"));

        // Test with an invalid prefix (empty)
        let result = UniqueIdentifier::new_with_prefix(prefix2);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to create Factory for xid: Failed to decode into a XID. Failed to retrieve the prefix from the provided encoded PXID "
        );

        // Test with an invalid prefix (contains space)
        let result = UniqueIdentifier::new_with_prefix(prefix3);
        assert!(result.is_err());
    }
}
