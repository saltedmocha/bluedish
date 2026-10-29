use serde::de::Error;
use thiserror::Error;

use crate::resp::ParsingError::TooShort;

const TYPE_IDENTIFIER: [u8; 15] = *b"+-:$*_#,(!=%|~>";

pub struct Resp(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParsingError {
    #[error("Prepend must be one of the reserved data type")]
    InvalidPrependType,
    #[error("Array/Bulk String must be prepended by data type char")]
    MissingPrependType,
    #[error("Must ends with carriage returns and newline")]
    MissingTerminator,
    #[error("Array/Bulk String contains non utf-8 character")]
    NonUTFCharacters,
    #[error("Input missing content")]
    TooShort,
    #[error("Bulk data type must include length")]
    MissingLength,
}

pub struct BulkString {
    pub size: u32,
    pub content: Vec<String>,
}

impl Default for Resp {
    fn default() -> Self {
        Self(String::from("$-1\r\n"))
    }
}

impl PartialEq for Resp {
    fn eq(&self, other: &Self) -> bool {
        let left = &self.0;
        let right = &other.0;

        if left != right {
            return false;
        }

        true
    }
}

impl TryFrom<&[u8]> for Resp {
    type Error = ParsingError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self(String::from("*-1\r\n")));
        }

        if !value.ends_with(b"\r\n") {
            return Err(ParsingError::MissingTerminator);
        }

        // Unwrap is fine (i think) it's checked that it's not empty already
        if !TYPE_IDENTIFIER.contains(value.first().unwrap()) {
            return Err(ParsingError::MissingPrependType);
        }

        let Ok(byte_str) = str::from_utf8(value) else {
            return Err(ParsingError::NonUTFCharacters);
        };

        Ok(Self(byte_str.to_string()))
    }
}

impl<'a> TryFrom<&'a Resp> for &'a [u8] {
    type Error = ParsingError;

    fn try_from(value: &'a Resp) -> Result<Self, Self::Error> {
        let chars = &value.0.chars().next();
        let Some(first) = chars else {
            return Err(ParsingError::MissingPrependType);
        };

        let Ok(chars_u8) = u8::try_from(*first) else {
            return Err(ParsingError::NonUTFCharacters);
        };

        if !TYPE_IDENTIFIER.contains(&chars_u8) {
            return Err(ParsingError::InvalidPrependType);
        }

        if !&value.0.ends_with("\r\n") {
            return Err(ParsingError::MissingTerminator);
        }

        Ok(value.0.as_bytes())
    }
}

impl TryFrom<Vec<u8>> for Resp {
    type Error = ParsingError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self(String::from("*-1\r\n")));
        }

        if !value.ends_with(b"\r\n") {
            return Err(ParsingError::MissingTerminator);
        }

        // Unwrap is fine (i think) it's checked that it's not empty already
        if !TYPE_IDENTIFIER.contains(value.first().unwrap()) {
            return Err(ParsingError::MissingPrependType);
        }

        let Ok(byte_str) = String::from_utf8(value) else {
            return Err(ParsingError::NonUTFCharacters);
        };

        Ok(Self(byte_str))
    }
}

impl TryFrom<Resp> for BulkString {
    type Error = ParsingError;

    fn try_from(value: Resp) -> Result<Self, Self::Error> {
        if !value.0.starts_with('$') {
            return Err(ParsingError::InvalidPrependType);
        }

        let mut value_split = value.0.split_terminator("\r\n");
        // Skip $
        let _ = value_split.next();
        let count = {
            let Some(count_str) = value_split.next() else {
                return Err(ParsingError::TooShort);
            };

            let Ok(count_parse) = count_str.parse::<usize>() else {
                return Err(ParsingError::MissingLength);
            };

            count_parse
        };

        let val_vec = value_split
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        if val_vec.len() <= count {
            return Err(ParsingError::TooShort);
        }

        let Ok(u32_count) = count.try_into() else {
            todo!()
        };

        let bulk_string = Self {
            size: u32_count,
            content: val_vec,
        };

        Ok(bulk_string)
    }
}

impl TryFrom<BulkString> for Resp {
    type Error = ParsingError;

    fn try_from(value: BulkString) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tryfrom_byte_slice() {
        let slice: &[u8] = b"+Success\r\n";
        let converted = Resp::try_from(slice);
        assert!(converted.is_ok());
    }

    #[test]
    fn test_slices_tryinto_resp() {
        let slices: &[u8] = &[42, 69, 113, 117, 97, 108, 32, 84, 101, 115, 116, 13, 10];
        let binding = Resp(String::from("*Equal Test\r\n"));
        let converted: Result<&[u8], ParsingError> = (&binding).try_into();

        assert_eq!(slices, converted.unwrap());
    }
}
