use std::cell::RefCell;

use crate::chunk::BlorbChunk;
use crate::error::BlorbError;
use crate::types::BlorbType;

/// A reader for blorb files
pub struct BlorbReader {
    stream: BlorbStream,
    ridx: Vec<RsrcIndex>,
}

pub(crate) struct BlorbStream {
    bytes: Vec<u8>,
    cursor: RefCell<usize>,
}

pub(crate) struct RsrcIndex {
    usage: BlorbType,
    id: usize,
    offset: usize,
}

pub(crate) struct RsrcInfo {
    pub(crate) blorb_type: BlorbType,
    size: usize,
}

impl BlorbReader {
    /// Create a blorb file reader from a vec of bytes
    pub fn new(bytes: Vec<u8>) -> Result<Self, BlorbError> {
        let stream = BlorbStream {
            bytes,
            cursor: RefCell::new(0),
        };

        if !stream.next_chunk_is(BlorbType::Form) {
            return Err(BlorbError::InvalidFileType);
        }

        let _file_size = stream.read_chunk_size()?;
        if !stream.next_chunk_is(BlorbType::Ifrs) {
            return Err(BlorbError::InvalidFileType);
        }

        Ok(Self {
            stream,
            ridx: Vec::new(),
        })
    }

    /// Retrieve a resouce by Resource ID as defined in the RIdx chunk
    pub fn get_resource_by_id(&self, _id: usize) -> Result<BlorbChunk, BlorbError> {
        let RsrcInfo { blorb_type, size } = self.get_rsrc_info()?;
        let offset = 0; // *self.cursor.borrow();
        Ok(BlorbChunk::new(
            blorb_type,
            &self.stream.bytes[offset..offset + size],
        ))
    }

    pub(crate) fn get_rsrc_info(&self) -> Result<RsrcInfo, BlorbError> {
        let blorb_type = self.stream.read_chunk_type()?;
        let size: usize = self.stream.read_chunk_size()?;

        Ok(RsrcInfo { blorb_type, size })
    }
}

impl BlorbStream {
    fn next_chunk_is(&self, blorb_type: BlorbType) -> bool {
        if let Ok(read_type) = self.read_chunk_type() {
            blorb_type == read_type
        } else {
            false
        }
    }

    fn read_chunk_type(&self) -> Result<BlorbType, BlorbError> {
        let offset = *self.cursor.borrow();

        // TODO: check offset in range
        *self.cursor.borrow_mut() += 4;

        (&self.bytes[offset..offset + 4]).try_into()
    }

    fn read_chunk_size(&self) -> Result<usize, BlorbError> {
        let offset = *self.cursor.borrow();

        // TODO: check offset in range
        *self.cursor.borrow_mut() += 4;
        Ok((self.bytes[offset] as usize) << 24
            | (self.bytes[offset + 1] as usize) << 16
            | (self.bytes[offset + 2] as usize) << 8
            | (self.bytes[offset + 3]) as usize)
    }
}
