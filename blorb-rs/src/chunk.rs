use crate::types::*;

/// An IFRS chunk
pub struct BlorbChunk<'a> {
    bytes: &'a [u8],
    blorb_type: BlorbType,
}

impl<'a> BlorbChunk<'a> {
    pub(crate) fn new(blorb_type: BlorbType, bytes: &'a [u8]) -> BlorbChunk {
        Self { bytes, blorb_type }
    }
}
