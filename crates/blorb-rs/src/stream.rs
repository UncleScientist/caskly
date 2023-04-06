use std::cell::RefCell;

use crate::{chunk::BlorbChunk, error::BlorbError, types::BlorbType};

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
        &(self.bytes[offset..offset + size])
    }

    pub fn read_chunk(&self) -> Result<BlorbChunk, BlorbError> {
        let offset = *self.cursor.borrow();
        let blorb_type = self.read_chunk_type()?;
        let size = self.read_chunk_size()?;
        Ok(BlorbChunk::new(
            blorb_type,
            &(self.bytes[offset..offset + size]),
        ))
    }

    pub fn seek(&self, offset: usize) {
        // TODO: check range
        *self.cursor.borrow_mut() = offset;
    }

    pub fn next_chunk_is(&self, blorb_type: BlorbType) -> bool {
        if let Ok(read_type) = self.read_chunk_type() {
            println!("expecting {blorb_type:?}, read {read_type:?}");
            blorb_type == read_type
        } else {
            println!("unable to extract chunk type");
            false
        }
    }

    pub fn read_chunk_type(&self) -> Result<BlorbType, BlorbError> {
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
