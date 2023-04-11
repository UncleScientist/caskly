use crate::types::*;

/// An IFRS chunk
pub struct BlorbChunk<'a> {
    usage: Option<ResourceType>,
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

    pub(crate) fn with_usage(mut self, usage: ResourceType) -> Self {
        self.usage = Some(usage);
        self
    }
}
