use std::cell::RefCell;

use crate::{
    chunk::RawBlorbChunk,
    error::BlorbError,
    types::{BlorbType, ResourceType},
};

#[derive(Debug)]
pub(crate) struct BlorbStream {
    bytes: Vec<u8>,
    cursor: RefCell<usize>,
}

impl BlorbStream {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            cursor: RefCell::new(0),
        }
    }

    pub fn get_next_chunk(&self, size: usize) -> &[u8] {
        let offset = *self.cursor.borrow();
        *self.cursor.borrow_mut() += size + (size % 2);
        &(self.bytes[offset..offset + size])
    }

    pub fn read_chunk(&self) -> Result<RawBlorbChunk, BlorbError> {
        let offset = *self.cursor.borrow();
        let blorb_type = self.read_chunk_type()?;
        let size = self.read_chunk_size()?;

        Ok(RawBlorbChunk::new(
            blorb_type,
            &(self.bytes[offset + 8..offset + 8 + size]),
        ))
    }

    pub fn seek(&self, offset: usize) {
        // TODO: check range
        *self.cursor.borrow_mut() = offset;
    }

    pub fn _get_offset(&self) -> usize {
        *self.cursor.borrow()
    }

    pub fn next_chunk_is(&self, blorb_type: BlorbType) -> bool {
        if let Ok(read_type) = self.read_chunk_type() {
            blorb_type == read_type
        } else {
            false
        }
    }

    pub fn read_chunk_type(&self) -> Result<BlorbType, BlorbError> {
        let offset = *self.cursor.borrow();

        if offset + 4 >= self.bytes.len() {
            return Err(BlorbError::EndOfFile);
        }

        *self.cursor.borrow_mut() += 4;

        (&self.bytes[offset..offset + 4]).try_into()
    }

    pub fn read_resource_type(&self) -> Result<ResourceType, BlorbError> {
        let offset = *self.cursor.borrow();

        // TODO: check offset in range
        *self.cursor.borrow_mut() += 4;

        (&self.bytes[offset..offset + 4]).try_into()
    }

    pub fn read_chunk_size(&self) -> Result<usize, BlorbError> {
        let offset = *self.cursor.borrow();

        // TODO: check offset in range
        *self.cursor.borrow_mut() += 4;
        Ok((self.bytes[offset] as usize) << 24
            | (self.bytes[offset + 1] as usize) << 16
            | (self.bytes[offset + 2] as usize) << 8
            | (self.bytes[offset + 3]) as usize)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_even_number_of_bytes() {
        let stream = BlorbStream::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let _ = stream.get_next_chunk(4);
        assert_eq!(*stream.cursor.borrow(), 4);
    }

    #[test]
    fn read_odd_number_of_bytes() {
        let stream = BlorbStream::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let _ = stream.get_next_chunk(3);
        assert_eq!(*stream.cursor.borrow(), 4);
    }
}
