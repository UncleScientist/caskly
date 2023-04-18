use crate::chunk::{BlorbChunk, RawBlorbChunk};
use crate::error::BlorbError;
use crate::stream::BlorbStream;
use crate::types::{BlorbType, ResourceType};

/// A reader for blorb files
#[derive(Debug)]
pub struct BlorbReader {
    stream: BlorbStream,
    ridx: Vec<RsrcIndex>,
}

#[derive(Debug)]
pub(crate) struct RsrcIndex {
    usage: ResourceType,
    id: usize,
    offset: usize,
}

/*
#[derive(Debug)]
pub(crate) struct RsrcInfo {
    resource_type: ResourceType,
    size: usize,
}
*/

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
            let usage = stream.read_resource_type()?;
            let id = stream.read_chunk_size()?;
            let offset = stream.read_chunk_size()?;
            ridx.push(RsrcIndex { usage, id, offset });
        }

        Ok(Self { stream, ridx })
    }

    /// Display a resource information entry
    pub fn dump_rsrc_usage(&self) {
        println!("{:?}", self.ridx);
    }

    /// Retrieve a resouce by Resource ID as defined in the RIdx chunk
    pub fn get_resource_by_id(&self, id: usize) -> Result<RawBlorbChunk, BlorbError> {
        for rsrc in &self.ridx {
            if rsrc.id == id {
                let offset = rsrc.offset;
                self.stream.seek(offset);
                return Ok(self.stream.read_chunk()?.with_usage(rsrc.usage));
            }
        }
        Err(BlorbError::NonExistentResource(id))
    }

    /// An iterator over chunks of a given type
    pub fn iter_type(&self, blorb_type: BlorbType) -> BlorbTypeIterator {
        self.stream.seek(12);
        BlorbTypeIterator {
            blorb: self,
            blorb_type,
        }
    }

    /// Convenience function to retrieve the first entry from an iterator
    pub fn get_first_rsrc_by_type(&self, blorb_type: BlorbType) -> Option<BlorbChunk> {
        let next_by_type = &self.iter_type(blorb_type).next()?;
        match next_by_type {
            Ok(next) => next.try_into().ok(),
            Err(_) => None,
        }
    }

    /*
    pub(crate) fn get_rsrc_info(&self) -> Result<RsrcInfo, BlorbError> {
        let resource_type = self.stream.read_resource_type()?;
        let size: usize = self.stream.read_chunk_size()?;

        Ok(RsrcInfo {
            resource_type,
            size,
        })
    }
    */

    pub(crate) fn read_next_chunk(&self) -> Result<RawBlorbChunk, BlorbError> {
        let blorb_type = self.stream.read_chunk_type()?;
        let chunk_size = self.stream.read_chunk_size()?;
        Ok(RawBlorbChunk::new(
            blorb_type,
            self.stream.get_next_chunk(chunk_size),
        ))
    }

    /// Returns an iterator which walks all of the chunks in a blorb file
    pub fn iter(&self) -> BlorbIterator {
        self.stream.seek(12);
        BlorbIterator { blorb: self }
    }
}

/// An iterator over all the chunks in a blorb file
pub struct BlorbIterator<'a> {
    blorb: &'a BlorbReader,
}

impl<'a> Iterator for BlorbIterator<'a> {
    type Item = Result<RawBlorbChunk<'a>, BlorbError>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.blorb.read_next_chunk() {
            Ok(chunk) => Some(Ok(chunk)),
            Err(BlorbError::EndOfFile) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Iterator for a specific type of resource
pub struct BlorbTypeIterator<'a> {
    blorb: &'a BlorbReader,
    blorb_type: BlorbType,
}

impl<'a> Iterator for BlorbTypeIterator<'a> {
    type Item = Result<RawBlorbChunk<'a>, BlorbError>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        loop {
            match self.blorb.read_next_chunk() {
                Ok(chunk) if chunk.blorb_type == self.blorb_type => {
                    return Some(Ok(chunk));
                }
                Err(BlorbError::EndOfFile) => return None,
                Err(e) => return Some(Err(e)),
                _ => {}
            }
        }
    }
}
