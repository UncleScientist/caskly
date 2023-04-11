use anyhow::Result;

use crate::error::BlorbError;

/// The IFF types that exist for blorb files
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum BlorbType {
    /// "FORM" - specifies that the file is an IFF type file
    Form,
    /// "IFRS" - the FORM type for blorb files
    Ifrs,
    /// "RIdx" - a Resource Index chunk
    Ridx,
    /// "Exec" - an RIDx usage type, executable chunk
    Exec,
    /// "Pict" - an RIDx usage type, an image chunk
    Pict,
}

macro_rules! blorb_type_try_from {
    ($type:ident, $($blorbType:ident => $string:expr),*) => {
        impl TryFrom<String> for $type {
            type Error = BlorbError;

            fn try_from(s: String) -> Result<Self, BlorbError> {
                match s.as_str() {
                    $($string => Ok(Self::$blorbType),)*
                    _ => Err(BlorbError::InvalidResourceType(s)),
                }
            }
        }

        impl TryFrom<&[u8]> for $type {
            type Error = BlorbError;

            fn try_from(t: &[u8]) -> Result<Self, BlorbError> {
                $(const $blorbType: &'static [u8] = $string.as_bytes();)*
                match t {
                    $($blorbType => Ok(Self::$blorbType),)*
                    _ => Err(BlorbError::InvalidResourceType(format!("given: {t:?}"))),
                }
            }
        }
    }
}

blorb_type_try_from!(
    BlorbType,
    Form => "FORM",
    Ifrs => "IFRS",
    Ridx => "RIdx",
    Exec => "Exec",
    Pict => "Pict"
);

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

    #[test]
    fn can_read_u8_ifrs() {
        assert_eq!(
            Ok(BlorbType::Ifrs),
            [b'I', b'F', b'R', b'S'][0..4].try_into()
        );
    }
}
