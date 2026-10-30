use thiserror::Error;

const TYPE_IDENTIFIER: [u8; 15] = *b"+-:$*_#,(!=%|~>";

#[derive(Debug)]
pub struct Resp(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParsingError {
    #[error("Input contains invalid data for prefix type")]
    InvalidContent,
    #[error("Prefix must be one of the reserved data type")]
    InvalidPrefixType,
    #[error("Bulk data type must include length")]
    MissingLength,
    #[error("Array/Bulk String must be prepended by data type char")]
    MissingPrefixType,
    #[error("Must ends with carriage returns and newline")]
    MissingTerminator,
    #[error("Array/Bulk String contains non utf-8 character")]
    NonUTFCharacters,
    #[error("Input missing content")]
    TooShort,
}

#[derive(Debug)]
pub struct BulkString {
    pub size: u32,
    pub content: Vec<String>,
}

impl Default for Resp {
    fn default() -> Self {
        Self("$-1\r\n".to_string())
    }
}

impl From<BulkString> for Resp {
    fn from(value: BulkString) -> Self {
        let content_string = value.content.join("\r\n");

        Self(format!("${}\r\n{}\r\n", value.size, content_string))
    }
}

impl From<i64> for Resp {
    fn from(value: i64) -> Self {
        Self(format!(":{value}\r\n"))
    }
}

impl From<Resp> for Vec<u8> {
    fn from(value: Resp) -> Self {
        value.0.into_bytes()
    }
}

impl<'a> TryFrom<&'a Resp> for &'a [u8] {
    type Error = ParsingError;

    fn try_from(value: &'a Resp) -> Result<Self, Self::Error> {
        let chars = &value.0.chars().next();
        let Some(first) = chars else {
            return Err(ParsingError::MissingPrefixType);
        };

        let Ok(chars_u8) = u8::try_from(*first) else {
            return Err(ParsingError::NonUTFCharacters);
        };

        if !TYPE_IDENTIFIER.contains(&chars_u8) {
            return Err(ParsingError::InvalidPrefixType);
        }

        if !&value.0.ends_with("\r\n") {
            return Err(ParsingError::MissingTerminator);
        }

        Ok(value.0.as_bytes())
    }
}

impl TryFrom<&[u8]> for Resp {
    type Error = ParsingError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self("*-1\r\n".to_string()));
        }

        if !value.ends_with(b"\r\n") {
            return Err(ParsingError::MissingTerminator);
        }

        // Unwrap is fine (i think) it's checked that it's not empty already
        if !TYPE_IDENTIFIER.contains(value.first().unwrap()) {
            return Err(ParsingError::MissingPrefixType);
        }

        let Ok(byte_str) = str::from_utf8(value) else {
            return Err(ParsingError::NonUTFCharacters);
        };

        Ok(Self(byte_str.to_string()))
    }
}

impl TryFrom<Resp> for i64 {
    type Error = ParsingError;

    fn try_from(value: Resp) -> Result<Self, Self::Error> {
        if !value.0.starts_with(':') {
            return Err(ParsingError::InvalidPrefixType);
        }

        let trimmed = {
            let base = &value.0;
            let trim_start = if base.contains('+') {
                base.trim_start_matches(":+")
            } else {
                base.trim_start_matches(':')
            };

            trim_start.trim_end_matches("\r\n")
        };

        let Ok(parsed) = trimmed.parse::<Self>() else {
            return Err(ParsingError::InvalidContent);
        };

        Ok(parsed)
    }
}

impl TryFrom<Resp> for BulkString {
    type Error = ParsingError;

    fn try_from(value: Resp) -> Result<Self, Self::Error> {
        let val_str = &value.0;

        // $-1\r\n (the bulk string null) is about 5 char long
        // \r and \n count as a single char each
        // Even in case of there being only 0 item
        // It'll still be at least 6 char long
        // For example: $0\r\n\r\n = $, 1, \r, \n, \r, \n,
        if val_str.len() < 5 {
            return Err(ParsingError::TooShort);
        }

        if !val_str.starts_with('$') {
            return Err(ParsingError::InvalidPrefixType);
        }

        if !val_str.ends_with("\r\n") {
            return Err(ParsingError::MissingTerminator);
        }

        let mut val_split = val_str.trim_start_matches('$').split("\r\n");
        let Some(count) = val_split.next() else {
            return Err(ParsingError::MissingLength);
        };

        let Ok(parsed_count) = count.parse::<u32>() else {
            return Err(ParsingError::MissingLength);
        };

        let mut val_vec = val_split
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        // Due to \r\n the split there will be always a empty string at the end
        val_vec.pop_if(|s| s.is_empty());

        Ok(Self {
            size: parsed_count,
            content: val_vec,
        })
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
            return Err(ParsingError::MissingPrefixType);
        }

        let Ok(byte_str) = String::from_utf8(value) else {
            return Err(ParsingError::NonUTFCharacters);
        };

        Ok(Self(byte_str))
    }
}

impl PartialEq for BulkString {
    fn eq(&self, other: &Self) -> bool {
        if self.size == other.size && self.content == other.content {
            return true;
        }

        false
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

impl TryFrom<Resp> for ErrorMessage {
    type Error = ParsingError;

    fn try_from(value: Resp) -> Result<Self, Self::Error> {
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

    #[test]
    fn test_tryfrom_resp_to_bulk() {
        let base = BulkString {
            size: 5,
            content: ["Test-1", "Test-2", "Test-3", "Test-4", "Test-5"]
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>(),
        };
        let test_in = Resp("$5\r\nTest-1\r\nTest-2\r\nTest-3\r\nTest-4\r\nTest-5\r\n".to_string());
        let result = BulkString::try_from(test_in).unwrap();

        assert_eq!(result, base);
    }

    #[test]
    fn test_from_bulk_to_resp() {
        let base = Resp("$0\r\n\r\n".to_string());
        let test_in = BulkString {
            size: 0,
            content: Vec::new(),
        };
        let result = Resp::from(test_in);

        assert_eq!(result, base);
    }

    #[test]
    fn test_from_resp_to_i64() {
        let base = 10;
        let test_in = Resp(":+10\r\n".to_string());
        let result = i64::try_from(test_in).unwrap();

        assert_eq!(result, base);
    }

    #[test]
    fn test_from_resp_to_negative_i64() {
        let base = -10;
        let test_in = Resp(":-10\r\n".to_string());
        let result = i64::try_from(test_in).unwrap();

        assert_eq!(result, base);
    }

    #[test]
    fn test_from_resp_to_plain_i64() {
        let base = 10;
        let test_in = Resp(":10\r\n".to_string());
        let result = i64::try_from(test_in).unwrap();

        assert_eq!(result, base);
    }

    #[test]
    fn test_text_on_resp_to_i64() {
        let err_type = ParsingError::InvalidPrefixType;
        let test_in = Resp("+40\r\n".to_string());
        let result = i64::try_from(test_in).unwrap_err();

        assert_eq!(result, err_type);
    }
}
