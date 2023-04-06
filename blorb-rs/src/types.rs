use anyhow::Result;

use crate::error::BlorbError;

/// The IFF types that exist for blorb files
#[derive(PartialEq, Debug)]
pub enum BlorbType {
    /// "FORM" - specifies that the file is an IFF type file
    Form,
    /// "IFRS" - the FORM type for blorb files
    Ifrs,
    /// "RIdx" - a Resource Index chunk
    Ridx,
}

impl TryFrom<String> for BlorbType {
    type Error = BlorbError;

    fn try_from(s: String) -> Result<Self, BlorbError> {
        match s.as_str() {
            "IFRS" => Ok(Self::Ifrs),
            "RIdx" => Ok(Self::Ridx),
            _ => Err(BlorbError::InvalidResourceType(s)),
        }
    }
}

impl TryFrom<&[u8]> for BlorbType {
    type Error = BlorbError;

    fn try_from(t: &[u8]) -> Result<Self, BlorbError> {
        match t {
            b"IFRS" => Ok(Self::Ifrs),
            b"RIdx" => Ok(Self::Ridx),
            _ => Err(BlorbError::InvalidResourceType(
                std::str::from_utf8(t).unwrap().to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_read_ifrs() {
        assert_eq!(Ok(BlorbType::Ifrs), "IFRS".to_string().try_into());
    }

    #[test]
    fn can_read_ridx() {
        assert_eq!(Ok(BlorbType::Ridx), "RIdx".to_string().try_into());
    }

    #[test]
    fn fails_on_invalid_input() {
        assert!(TryInto::<BlorbType>::try_into("asdflkjasdf".to_string()).is_err());
    }
}