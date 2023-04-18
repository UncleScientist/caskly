use std::fmt::{Debug, Formatter};

use crate::{error::BlorbError, types::*};

/// A raw IFRS chunk
pub struct RawBlorbChunk<'a> {
    usage: Option<ResourceType>,
    /// The type of data stored in the bytes field
    pub blorb_type: BlorbType,
    /// Raw data from the blorb file
    pub bytes: &'a [u8],
}

/// Decoded chunk information
#[derive(Debug)]
pub enum BlorbChunk {
    /// An Fspc resource chunk
    Frontispiece(usize),

    /// A resource description chunk
    ResourceDescription(Vec<TextDescription>),
}

/// A textual description of a visual or auditory resource
#[derive(Debug)]
pub struct TextDescription {
    /// Resource Usage
    pub usage: ResourceType,
    /// Number of resource
    pub number: usize,
    /// Textual description
    pub text: String,
}

impl<'a> RawBlorbChunk<'a> {
    pub(crate) fn new(blorb_type: BlorbType, bytes: &'a [u8]) -> RawBlorbChunk {
        Self {
            usage: None,
            blorb_type,
            bytes,
        }
    }

    pub(crate) fn with_usage(mut self, usage: ResourceType) -> Self {
        self.usage = Some(usage);
        self
    }
}

impl<'a> Debug for RawBlorbChunk<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{{ usage = {}",
            if let Some(u) = self.usage {
                format!("Some({u:?})")
            } else {
                "None".to_string()
            }
        )?;
        write!(f, ", blorb_type = {:?}, [ ", self.blorb_type)?;
        for (i, b) in self.bytes.iter().enumerate().take(4) {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{b}")?;
        }
        if self.bytes.len() > 4 {
            write!(f, ", ... ] }}")?;
        } else {
            write!(f, " ] }}")?;
        }
        Ok(())
    }
}

impl<'a> TryFrom<&RawBlorbChunk<'a>> for BlorbChunk {
    type Error = BlorbError;

    fn try_from(bc: &RawBlorbChunk<'a>) -> Result<Self, BlorbError> {
        match bc.blorb_type {
            BlorbType::Fspc => Ok(Self::Frontispiece(bytes_to_usize(bc.bytes)?)),
            BlorbType::Rdes => {
                let mut entries = Vec::new();
                let mut offset = 4;
                for _ in 0..bytes_to_usize(&bc.bytes[0..4])? {
                    let usage: ResourceType = bc.bytes[offset..offset + 4].try_into()?;
                    let number = bytes_to_usize(&bc.bytes[offset + 4..offset + 8])?;
                    let len = bytes_to_usize(&bc.bytes[offset + 8..offset + 12])?;
                    let text = std::str::from_utf8(&bc.bytes[offset + 12..offset + 12 + len])
                        .map_err(|_| BlorbError::InvalidUtf8String)?
                        .to_string();
                    entries.push(TextDescription {
                        usage,
                        number,
                        text,
                    });
                    offset += 12 + len;
                }
                Ok(Self::ResourceDescription(entries))
            }
            _ => Err(BlorbError::ConversionFailed),
        }
    }
}

fn bytes_to_usize(bytes: &[u8]) -> Result<usize, BlorbError> {
    if bytes.len() != 4 {
        Err(BlorbError::ConversionFailed)
    } else {
        // TODO: refactor with BlorbReader's version
        Ok((bytes[0] as usize) << 24
            | (bytes[1] as usize) << 16
            | (bytes[2] as usize) << 8
            | (bytes[3] as usize))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_read_rdes_data() {
        let bytes: [u8; 54] = [
            0, 0, 0, 2, 0x50, 0x69, 0x63, 0x74, 0, 0, 0, 3, 0, 0, 0, 0xd, 0x64, 0x69, 0x6d, 0x20,
            0x6e, 0x6f, 0x72, 0x74, 0x68, 0x77, 0x65, 0x73, 0x74, 0x50, 0x69, 0x63, 0x74, 0, 0, 0,
            4, 0, 0, 0, 0xd, 0x64, 0x69, 0x6d, 0x20, 0x6e, 0x6f, 0x72, 0x74, 0x68, 0x77, 0x65,
            0x73, 0x74,
        ];

        let rbc = RawBlorbChunk {
            usage: None,
            blorb_type: BlorbType::Rdes,
            bytes: &bytes,
        };
        let rdes: BlorbChunk = (&rbc).try_into().expect("could not convert");
        match rdes {
            BlorbChunk::ResourceDescription(v) => assert_eq!(v.len(), 2),
            _ => panic!("invalid conversion"),
        }
    }

    fn implements_debug<T: Debug>() {}

    #[test]
    fn chunk_can_generate_debug_output() {
        implements_debug::<RawBlorbChunk>();
    }
}
