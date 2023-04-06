use crate::types::*;

/// An IFRS chunk
pub struct BlorbChunk<'a> {
    usage: Option<BlorbType>,
    blorb_type: BlorbType,
    bytes: &'a [u8],
}

impl<'a> BlorbChunk<'a> {
    pub(crate) fn new(blorb_type: BlorbType, bytes: &'a [u8]) -> BlorbChunk {
        Self {
            usage: None,
            blorb_type,
            bytes,
        }
    }

    pub(crate) fn with_usage(mut self, usage: BlorbType) -> Self {
        self.usage = Some(usage);
        self
    }
}
