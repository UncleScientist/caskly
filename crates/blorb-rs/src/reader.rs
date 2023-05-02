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

    fn is_type(chunk: &Result<RawBlorbChunk, BlorbError>, blorb_type: BlorbType) -> bool {
        if let Ok(chunk) = chunk {
            chunk.blorb_type == blorb_type
        } else {
            false
        }
    }

    pub(crate) fn find_chunk(&self, blorb_type: BlorbType) -> Result<RawBlorbChunk, BlorbError> {
        self.iter()
            .filter(|chunk| Self::is_type(chunk, blorb_type))
            .next()
            .ok_or(BlorbError::ChunkNotFound)?
    }

    /// Retrieve the image associated with the Frontispiece chunk
    pub fn get_frontispiece_image(&self) -> Option<RawBlorbChunk> {
        let chunk = self.find_chunk(BlorbType::Fspc).ok()?;
        if let BlorbChunk::Frontispiece(num) = (&chunk).try_into().ok()? {
            return self.get_resource(ResourceType::Pict, num).ok();
        }

        None
    }

    /// Retrieve the game idenfier chunk
    pub fn get_game_identifier(&self) -> Option<BlorbChunk> {
        if let Ok(chunk) = self.find_chunk(BlorbType::Ifhd) {
            (&chunk).try_into().ok()
        } else {
            None
        }
    }

    /// Display a resource information entry
    pub fn dump_rsrc_usage(&self) {
        println!("{:?}", self.ridx);
    }

    /// Retrieve a resouce by Resource ID as defined in the RIdx chunk
    pub fn get_resource(
        &self,
        usage: ResourceType,
        id: usize,
    ) -> Result<RawBlorbChunk, BlorbError> {
        for rsrc in &self.ridx {
            if rsrc.id == id && rsrc.usage == usage {
                let offset = rsrc.offset;
                self.stream.seek(offset);
                return Ok(self.stream.read_chunk()?.with_usage(rsrc.usage));
            }
        }
        Err(BlorbError::NonExistentResource(id))
    }

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
