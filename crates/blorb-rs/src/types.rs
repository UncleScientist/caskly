use anyhow::Result;
use paste::paste;

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
    /// A Glulx executable chunk
    Glul,
    /// Blorb file metadata
    Ifmd,
    /// A Fronispiece chunk
    Fspc,
    /// A PNG image chunk
    Png,
    /// A JPeg image chunk
    Jpeg,
    /// An Ogg Vorbis sound chunk
    Oggv,
}

/// In the RIdx chunk, the file defines four different types of resources
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ResourceType {
    /// "Pict" - an image resource
    Pict,
    /// "Snd " - a sound resource
    Sound,
    /// "Data" - a chunk of data
    Data,
    /// "Exec" - the executable game to play
    Executable,
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
                paste! {
                    $(const [<$blorbType:upper>] : &'static [u8] = $string.as_bytes();)*
                    match t {
                        $([<$blorbType:upper>] => Ok(Self::$blorbType),)*
                        _ => {
                            let mut s = String::new();
                            for byte in t {
                                if *byte >= 32 && *byte <= 126 {
                                    s.push(*byte as char);
                                } else {
                                    s.push_str(format!("'0x{byte}'").as_str());
                                }
                            }
                            Err(BlorbError::InvalidResourceType(format!("given: \"{s}\"")))
                        }
                    }
                }
            }
        }
    }
}

blorb_type_try_from!(
    ResourceType,
    Pict => "Pict",
    Sound => "Snd ",
    Data => "Data",
    Executable => "Exec"
);

blorb_type_try_from!(
    BlorbType,
    Form => "FORM",
    Ifrs => "IFRS",
    Ridx => "RIdx",
    Ifmd => "IFmd",
    Fspc => "Fspc",
    Png => "PNG ",
    Jpeg => "JPEG",
    Glul => "GLUL",
    Oggv => "OGGV"
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

    #[test]
    fn can_read_pict_resource_type() {
        assert_eq!(Ok(ResourceType::Pict), "Pict".to_string().try_into());
    }
}
