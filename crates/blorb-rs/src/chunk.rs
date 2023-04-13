use std::fmt::{Debug, Formatter};

use crate::{error::BlorbError, types::*};

/// A raw IFRS chunk
pub struct BlorbChunk<'a> {
    usage: Option<ResourceType>,
    /// The type of data stored in the bytes field
    pub blorb_type: BlorbType,
    /// Raw data from the blorb file
    pub bytes: &'a [u8],
}

/// Decoded chunk information
pub enum Chunk {
    /// An Fspc resource chunk
    Frontispiece(usize),
}

impl<'a> BlorbChunk<'a> {
    pub(crate) fn new(blorb_type: BlorbType, bytes: &'a [u8]) -> BlorbChunk {
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

impl<'a> Debug for BlorbChunk<'a> {
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

#[cfg(test)]
mod test {
    use super::*;

    fn implements_debug<T: Debug>() {}

    #[test]
    fn chunk_can_generate_debug_output() {
        implements_debug::<BlorbChunk>();
    }
}

impl<'a> TryFrom<&BlorbChunk<'a>> for Chunk {
    type Error = BlorbError;

    fn try_from(bc: &BlorbChunk<'a>) -> Result<Self, BlorbError> {
        match bc.blorb_type {
            BlorbType::Fspc => Ok(Self::Frontispiece(bytes_to_usize(bc.bytes)?)),
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
