use thiserror::Error;

const TYPE_IDENTIFIER: [u8; 15] = *b"+-:$*_#,(!=%|~>";

pub struct Resp(String);

#[derive(Debug, Error, PartialEq)]
pub enum RespError {
    #[error("Array/Bulk String must be prepended by data type char")]
    MissingPrependType,
    #[error("Must ends with carriage returns and newline")]
    MissingTerminator,
    #[error("Prepend must be one of the reserved data type")]
    InvalidPrependType,
    #[error("Array/Bulk String contains non utf-8 character")]
    NonUTFCharacters,
}

impl Default for Resp {
    fn default() -> Self {
        Resp(String::from("$-1\r\n"))
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
    type Error = RespError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Resp(String::from("*-1\r\n")));
        }

        if !value.ends_with("\r\n".as_bytes()) {
            return Err(RespError::MissingTerminator);
        }

        // Unwrap is fine (i think) it's checked that it's not empty already
        if !TYPE_IDENTIFIER.contains(value.first().unwrap()) {
            return Err(RespError::MissingPrependType);
        }

        let Ok(byte_str) = str::from_utf8(value) else {
            return Err(RespError::NonUTFCharacters);
        };

        Ok(Resp(byte_str.to_string()))
    }
}

impl<'a> TryFrom<&'a Resp> for &'a [u8] {
    type Error = RespError;

    fn try_from(value: &'a Resp) -> Result<Self, Self::Error> {
        let chars = &value.0.chars().next();
        let Some(first) = chars else {
            return Err(RespError::MissingPrependType);
        };

        let Ok(chars_u8) = u8::try_from(*first) else {
            return Err(RespError::NonUTFCharacters);
        };

        if !TYPE_IDENTIFIER.contains(&chars_u8) {
            return Err(RespError::InvalidPrependType);
        }

        if !&value.0.ends_with("\r\n") {
            return Err(RespError::MissingTerminator);
        }

        Ok(value.0.as_bytes())
    }
}

impl TryFrom<Vec<u8>> for Resp {
    type Error = RespError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Resp(String::from("*-1\r\n")));
        }

        if !value.ends_with("\r\n".as_bytes()) {
            return Err(RespError::MissingTerminator);
        }

        // Unwrap is fine (i think) it's checked that it's not empty already
        if !TYPE_IDENTIFIER.contains(value.first().unwrap()) {
            return Err(RespError::MissingPrependType);
        }

        let Ok(byte_str) = String::from_utf8(value) else {
            return Err(RespError::NonUTFCharacters);
        };

        Ok(Resp(byte_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tryfrom_byte_slice() {
        let slice = "+Success\r\n".as_bytes();
        let converted = Resp::try_from(slice);
        assert!(converted.is_ok());
    }

    #[test]
    fn test_slices_tryinto_resp() {
        let slices: &[u8] = &[42, 69, 113, 117, 97, 108, 32, 84, 101, 115, 116, 13, 10];
        let binding = Resp(String::from("*Equal Test\r\n"));
        let converted: Result<&[u8], RespError> = (&binding).try_into();

        assert_eq!(slices, converted.unwrap())
    }
}
