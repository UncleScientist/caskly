use crate::types::*;

/// An IFRS chunk
#[derive(Debug)]
pub struct BlorbChunk<'a> {
    usage: Option<ResourceType>,
    /// The type of data stored in the bytes field
    pub blorb_type: BlorbType,
    /// Raw data from the blorb file
    pub bytes: &'a [u8],
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

#[cfg(test)]
mod test {
    use super::*;

    fn implements_debug<T: std::fmt::Debug>() {}

    #[test]
    fn chunk_can_generate_debug_output() {
        implements_debug::<BlorbChunk>();
    }
}
