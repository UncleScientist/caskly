use anyhow::Result;

use crate::error::BlorbError;

#[derive(PartialEq, Debug)]
pub enum BlorbType {
    Ifrs, // 'IFRS'
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
