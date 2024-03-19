use std::fmt;
use thiserror::Error;

/// Error type for Prefix creation failures.
#[derive(Error, Debug, PartialEq)]
pub enum PrefixError {
    #[error("Invalid prefix length: Expected 1 to 4 bytes. Received: {0}")]
    InvalidPrefixLength(String),
    #[error("Invalid UTF-8 input")]
    InvalidUtf8,
}

/// Generates a 4-byte string slice prefix from input string. If the input
/// string is shorter than 4 bytes, it pads the result with zeros.
fn prefix(input: &str) -> Result<[u8; 4], PrefixError> {
    if !std::str::from_utf8(input.as_bytes()).is_ok() {
        return Err(PrefixError::InvalidUtf8);
    }

    let bytes = input.as_bytes();
    let len = bytes.len();
    if len == 0 || len > 4 {
        return Err(PrefixError::InvalidPrefixLength(format!(
            "Expected 1 to 4 bytes. Received: {}",
            len
        )));
    }
    let mut result = [0; 4];
    result[..len].copy_from_slice(&bytes[..len]);
    Ok(result)
}

/// Represents a concatenated ID consisting of a prefix and a timestamp.
#[derive(Debug)]
pub struct GlobalId([u8; 12]);

impl GlobalId {
    /// Generates a concatenated ID using the given prefix string.
    /// The provided prefix must not exceed 4 bytes in length. Prefix is a way to provide
    /// human readeable context to a global ID, similar to a `tag`.
    ///
    /// ```ignore
    /// V V V V  W W W W W W W W
    /// └─────┘ └───────────────┘
    ///    |           |
    ///  Prefix    Timestamp
    /// ```
    pub fn new(pfx: &str) -> [u8; 12] {
        let id_prefix = prefix(pfx).unwrap_or_else(|err| {
            panic!("Failed to generate prefix: {}", err);
        });
        let timestamp = crate::utils::timestamp();
        let mut global_id = [0; 12];

        // Fill the first four elements with the prefix bytes
        global_id[..4].copy_from_slice(&id_prefix);
        // Fill the next eight elements with the timestamp bytes
        global_id[4..].copy_from_slice(&timestamp.to_be_bytes());

        global_id
    }

    /// Converts the GlobalID to a Vec<u8>, convenience method to satisfy Structsy, ID type.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
    /// Converts a Vec<u8> to a GlobalId.
    ///
    /// This method expects the input vector to have a length of 12 bytes.
    /// If the length is different, it will panic.
    pub fn from_vec(vec: Vec<u8>) -> GlobalId {
        assert_eq!(vec.len(), 12, "Input vector length must be 12 bytes");

        let mut global_id = [0; 12];
        global_id.copy_from_slice(&vec);
        GlobalId(global_id)
    }

    /// Returns the prefix string from the GlobalId.
    #[allow(dead_code)]
    fn get_prefix_str(&self) -> String {
        // Extract the prefix bytes from the first 4 elements of the array
        let prefix_bytes = &self.0[..4];
        // Convert the bytes to characters and collect them into a string
        prefix_bytes.iter().map(|&b| b as char).collect()
    }

    /// Returns the timestamp from the GlobalId.
    pub fn get_timestamp(&self) -> u64 {
        // Extract the timestamp bytes from the remaining 8 elements of the array
        let timestamp_bytes = &self.0[4..];
        // Ensure that there are exactly 8 bytes for the timestamp
        assert_eq!(timestamp_bytes.len(), 8);
        // Convert the bytes to a u64 value using big-endian byte order
        u64::from_be_bytes([
            timestamp_bytes[0],
            timestamp_bytes[1],
            timestamp_bytes[2],
            timestamp_bytes[3],
            timestamp_bytes[4],
            timestamp_bytes[5],
            timestamp_bytes[6],
            timestamp_bytes[7],
        ])
    }
}

impl fmt::Display for GlobalId {
    /// Formats the GlobalId as a human-readable string.
    ///
    /// The first 4 bytes are represented as characters,
    /// remaining 8 bytes are represented as hexadecimal digits.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Represent bytes 0 through 3 as chars
        for &byte in &self.0[..4] {
            write!(f, "{}", char::from(byte).to_uppercase())?;
        }

        // Represent bytes 4 through 11 as hexadecimal string
        for &byte in &self.0[4..] {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use GlobalId;

    #[test]
    fn test_timestamp() {
        // Test if timestamp function returns a value greater than 0
        assert!(crate::utils::timestamp() > 0);
    }

    #[test]
    fn test_prefix_valid_length() {
        // Test prefix function with valid input length
        assert_eq!(prefix("abcd").unwrap(), [97, 98, 99, 100]); // ASCII values: 'a', 'b', 'c', 'd'
    }

    #[test]
    fn test_prefix_invalid_length() {
        // Test prefix function with invalid input length
        assert_eq!(
            prefix("abcdefghi").unwrap_err(),
            PrefixError::InvalidPrefixLength(String::from("Expected 1 to 4 bytes. Received: 9"))
        );
    }

    #[test]
    fn test_prefix_zero_length() {
        // Test prefix function with invalid input length
        assert_eq!(
            prefix("").unwrap_err(),
            PrefixError::InvalidPrefixLength(String::from("Expected 1 to 4 bytes. Received: 0"))
        );
    }

    #[test]
    fn test_concatenated_id() {
        // Test concatenated_id function
        let concatenated_id = GlobalId::new("test");
        assert_eq!(concatenated_id.len(), 12); // Length should be 12 bytes
    }

    #[test]
    fn test_display_to_uppercase() {
        // Test Display implementation for ConcatenatedId
        // first 4 bytes, prefix = "abcd"
        let concatenated_id = GlobalId([97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108]);
        assert_eq!(format!("{}", concatenated_id), "ABCD65666768696A6B6C");
    }

    #[test]
    fn test_to_vec() {
        let global_id = GlobalId([
            0x61, 0x62, 0x63, 0x64, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        ]);
        let vec = global_id.to_vec();
        assert_eq!(
            vec,
            vec![0x61, 0x62, 0x63, 0x64, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
        );
    }

    #[test]
    fn test_from_vec() {
        let vec = vec![
            0x61, 0x62, 0x63, 0x64, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        ];
        let global_id = GlobalId::from_vec(vec.clone());
        assert_eq!(
            global_id.0,
            [0x61, 0x62, 0x63, 0x64, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
        );
    }

    #[test]
    fn test_get_prefix_str() {
        let global_id = GlobalId([
            0x61, 0x62, 0x63, 0x64, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        ]);
        assert_eq!(global_id.get_prefix_str(), "abcd");
    }

    #[test]
    fn test_get_timestamp() {
        let mut gid: GlobalId = GlobalId([0; 12]);
        let timestamp: u64 = 1710778108;
        let bytes_conversion = timestamp.to_be_bytes();
        let prefix = prefix("test").unwrap();
        gid.0[..4].copy_from_slice(&prefix);
        gid.0[4..].copy_from_slice(&bytes_conversion);

        assert_eq!(gid.get_timestamp(), 1710778108);
    }
}
