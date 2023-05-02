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
#[derive(Debug, PartialEq)]
pub enum BlorbChunk {
    /// An Fspc resource chunk (Blorb Spec section 9)
    Frontispiece(usize),

    /// A resource description chunk (Blorb Spec section 9)
    ResourceDescription(Vec<TextDescription>),

    /// An author chunk (Blorb Spec section 12)
    Author(String),

    /// A copyright chunk (Blorb Spec section 12)
    Copyright(String),

    /// An annotation chunk (Blorb Spec section 12)
    Annotation(String),

    /// A "Rect" placeholder image (Blorb Spec section 2.3)
    Placeholder(usize, usize),

    /// A game identifier for Z-Code files
    GameIdentifier {
        /// release numer ($2 in header)
        release_number: u16,
        /// serial number ($12 in header)
        serial_number: [u8; 6],
        /// checksum ($1C in header)
        checksum: u16,
        /// Starting PC value
        pc: [u8; 3],
    },

    /// The release number of the resource file.
    ReleaseNumber(u16),

    /// A resolution chunk for scaling images
    Resolution {
        /// Standard window width and height
        standard: WindowSize,
        /// Minimum window width and height
        minimum: WindowSize,
        /// Maximum window width and height
        maximum: WindowSize,
        /// Image resolution entries
        entries: Vec<ResolutionEntry>,
    },

    /// A list of picture resources which have adaptive palette colors
    AdaptivePalette(Vec<usize>),
}

/// The size of a window for the resolution chunk
#[derive(Debug, PartialEq)]
pub struct WindowSize {
    width: usize,
    height: usize,
}

/// A resolution definition for an image resource
#[derive(Debug, PartialEq)]
pub struct ResolutionEntry {
    /// image resource number
    number: usize,
    /// Standard ratio numerator and denominator
    standard: ResolutionRatio,
    /// Minimum ratio numerator and denominator
    minimum: ResolutionRatio,
    /// Maximum ratio numerator and denominator
    maximum: ResolutionRatio,
}

/// A resolution ratio
#[derive(Debug, PartialEq)]
pub struct ResolutionRatio {
    /// numerator of the ratio
    numerator: usize,
    /// denominator of the ratio
    denominator: usize,
}

impl ResolutionRatio {
    /// Convert the ratio to a real number
    pub fn ratio(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

/// A textual description of a visual or auditory resource
#[derive(Debug, PartialEq)]
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

// TODO: look into using the binread crate to do the conversions for us
impl<'a> TryFrom<&RawBlorbChunk<'a>> for BlorbChunk {
    type Error = BlorbError;

    fn try_from(bc: &RawBlorbChunk<'a>) -> Result<Self, BlorbError> {
        match bc.blorb_type {
            BlorbType::Fspc => Ok(Self::Frontispiece(bytes_to_usize(bc.bytes)?)),
            BlorbType::Auth => Ok(Self::Author(bytes_to_string(bc.bytes)?)),
            BlorbType::Copr => Ok(Self::Copyright(bytes_to_string(bc.bytes)?)),
            BlorbType::Anno => Ok(Self::Annotation(bytes_to_string(bc.bytes)?)),
            BlorbType::Reln => Ok(Self::ReleaseNumber(bytes_to_u16(&bc.bytes[0..2])?)),
            BlorbType::Apal => {
                if bc.bytes.len() == 0 {
                    return Ok(Self::AdaptivePalette(Vec::new()));
                }

                let num = bytes_to_usize(&bc.bytes[0..4])?;
                if num % 4 != 0 {
                    return Err(BlorbError::ConversionFailed);
                }
                let mut entries = Vec::new();
                for i in 0..num % 4 {
                    let start = 4 + i * 4;
                    entries.push(bytes_to_usize(&bc.bytes[start..start + 4])?);
                }
                Ok(Self::AdaptivePalette(entries))
            }
            BlorbType::Ifhd => {
                if bc.bytes.len() != 13 {
                    return Err(BlorbError::ConversionFailed);
                }
                let mut serial_number = [0; 6];
                let mut pc = [0; 3];
                serial_number.clone_from_slice(&bc.bytes[2..8]);
                pc.clone_from_slice(&bc.bytes[10..13]);
                Ok(Self::GameIdentifier {
                    release_number: bytes_to_u16(&bc.bytes[0..2])?,
                    serial_number,
                    checksum: bytes_to_u16(&bc.bytes[8..10])?,
                    pc,
                })
            }
            BlorbType::Rect => {
                let width = bytes_to_usize(&bc.bytes[0..4])?;
                let height = bytes_to_usize(&bc.bytes[4..8])?;
                Ok(Self::Placeholder(width, height))
            }
            BlorbType::Rdes => {
                let mut entries = Vec::new();
                let mut offset = 4;
                for _ in 0..bytes_to_usize(&bc.bytes[0..4])? {
                    let usage: ResourceType = bc.bytes[offset..offset + 4].try_into()?;
                    let number = bytes_to_usize(&bc.bytes[offset + 4..offset + 8])?;
                    let len = bytes_to_usize(&bc.bytes[offset + 8..offset + 12])?;
                    let text = bytes_to_string(&bc.bytes[offset + 12..offset + 12 + len])?;
                    entries.push(TextDescription {
                        usage,
                        number,
                        text,
                    });
                    offset += 12 + len;
                }
                Ok(Self::ResourceDescription(entries))
            }
            BlorbType::Reso => {
                let entry_count = bc.bytes.len() - 24;
                if entry_count % 28 != 0 {
                    return Err(BlorbError::ConversionFailed);
                }

                let entry_count = entry_count / 28;

                let px = bytes_to_usize(&bc.bytes[0..4])?;
                let py = bytes_to_usize(&bc.bytes[4..8])?;
                let standard = WindowSize {
                    width: px,
                    height: py,
                };

                let minx = bytes_to_usize(&bc.bytes[8..12])?;
                let miny = bytes_to_usize(&bc.bytes[12..16])?;
                let minimum = WindowSize {
                    width: minx,
                    height: miny,
                };

                let maxx = bytes_to_usize(&bc.bytes[16..20])?;
                let maxy = bytes_to_usize(&bc.bytes[20..24])?;
                let maximum = WindowSize {
                    width: maxx,
                    height: maxy,
                };

                let mut entries = Vec::new();
                let mut offset = 4;
                for _ in 0..entry_count {
                    let number = bytes_to_usize(&bc.bytes[offset..offset + 4])?;
                    let ratnum = bytes_to_usize(&bc.bytes[offset + 4..offset + 8])?;
                    let ratden = bytes_to_usize(&bc.bytes[offset + 8..offset + 12])?;
                    let standard = ResolutionRatio {
                        numerator: ratnum,
                        denominator: ratden,
                    };

                    let minnum = bytes_to_usize(&bc.bytes[offset + 12..offset + 16])?;
                    let minden = bytes_to_usize(&bc.bytes[offset + 16..offset + 20])?;
                    let minimum = ResolutionRatio {
                        numerator: minnum,
                        denominator: minden,
                    };

                    let maxnum = bytes_to_usize(&bc.bytes[offset + 20..offset + 24])?;
                    let maxden = bytes_to_usize(&bc.bytes[offset + 24..offset + 28])?;
                    let maximum = ResolutionRatio {
                        numerator: maxnum,
                        denominator: maxden,
                    };
                    entries.push(ResolutionEntry {
                        number,
                        standard,
                        minimum,
                        maximum,
                    });
                    offset += 28;
                }
                Ok(Self::Resolution {
                    standard,
                    minimum,
                    maximum,
                    entries,
                })
            }
            _ => Err(BlorbError::ConversionFailed),
        }
    }
}

fn bytes_to_string(bytes: &[u8]) -> Result<String, BlorbError> {
    Ok(std::str::from_utf8(bytes)
        .map_err(|_| BlorbError::InvalidUtf8String)?
        .to_string())
}

fn bytes_to_u16(bytes: &[u8]) -> Result<u16, BlorbError> {
    if bytes.len() != 2 {
        Err(BlorbError::ConversionFailed)
    } else {
        Ok((bytes[0] as u16) << 8 | (bytes[1] as u16))
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
        let bytes = [
            0u8, 0, 0, 2, 0x50, 0x69, 0x63, 0x74, 0, 0, 0, 3, 0, 0, 0, 0xd, 0x64, 0x69, 0x6d, 0x20,
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

    #[test]
    fn can_interpret_rect_chunk() {
        let bytes = [0u8, 0, 1, 0, 0, 0, 2, 0];
        let rbc = RawBlorbChunk {
            usage: None,
            blorb_type: BlorbType::Rect,
            bytes: &bytes,
        };
        let rdes: BlorbChunk = (&rbc).try_into().expect("could not convert");
        assert_eq!(BlorbChunk::Placeholder(256, 512), rdes);
    }

    fn implements_debug<T: Debug>() {}

    #[test]
    fn chunk_can_generate_debug_output() {
        implements_debug::<RawBlorbChunk>();
    }
}
