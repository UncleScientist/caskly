use crate::chunk::BlorbChunk;
use crate::error::BlorbError;
use crate::stream::BlorbStream;
use crate::types::BlorbType;

/// A reader for blorb files
pub struct BlorbReader {
    stream: BlorbStream,
    ridx: Vec<RsrcIndex>,
}

#[derive(Debug)]
pub(crate) struct RsrcIndex {
    usage: BlorbType,
    id: usize,
    offset: usize,
}

#[derive(Debug)]
pub(crate) struct RsrcInfo {
    pub(crate) blorb_type: BlorbType,
    size: usize,
}

impl BlorbReader {
    /// Create a blorb file reader from a vec of bytes
    pub fn new(bytes: Vec<u8>) -> Result<Self, BlorbError> {
        let stream = BlorbStream::new(bytes);

        if !stream.next_chunk_is(BlorbType::Form) {
            return Err(BlorbError::InvalidFileType);
        }

        let _file_size = stream.read_chunk_size()?;
        if !stream.next_chunk_is(BlorbType::Ifrs) {
            return Err(BlorbError::InvalidFileType);
        }

        if !stream.next_chunk_is(BlorbType::Ridx) {
            return Err(BlorbError::InvalidFileType);
        }
        let _size = stream.read_chunk_size()?;
        let count = stream.read_chunk_size()?;

        let mut ridx = Vec::new();
        for _ in 0..count {
            let usage = stream.read_chunk_type()?;
            let id = stream.read_chunk_size()?;
            let offset = stream.read_chunk_size()?;
            ridx.push(RsrcIndex { usage, id, offset });
        }

        Ok(Self { stream, ridx })
    }

    pub fn dump_rsrc_usage(&self) {
        println!("{:?}", self.ridx);
    }

    /// Retrieve a resouce by Resource ID as defined in the RIdx chunk
    pub fn get_resource_by_id(&self, id: usize) -> Result<BlorbChunk, BlorbError> {
        for rsrc in &self.ridx {
            if rsrc.id == id {
                let offset = rsrc.offset;
                self.stream.seek(offset);
                return Ok(self.stream.read_chunk()?.with_usage(rsrc.usage));
            }
        }
        Err(BlorbError::NonExistentResource(id))
    }

    pub(crate) fn get_rsrc_info(&self) -> Result<RsrcInfo, BlorbError> {
        let blorb_type = self.stream.read_chunk_type()?;
        let size: usize = self.stream.read_chunk_size()?;

        Ok(RsrcInfo { blorb_type, size })
    }

    pub(crate) fn read_next_chunk(&self) -> Result<BlorbChunk, BlorbError> {
        let blorb_type = self.stream.read_chunk_type()?;
        let chunk_size = self.stream.read_chunk_size()?;
        Ok(BlorbChunk::new(
            blorb_type,
            self.stream.get_next_chunk(chunk_size),
        ))
    }
}
