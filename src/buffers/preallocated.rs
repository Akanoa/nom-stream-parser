use std::ops::{Deref, DerefMut};

use crate::debug;
use crate::errors::StreamParserError;
use crate::traits::Buffer;

/// A buffer of preallocated heap data
pub struct BufferPreallocated<'a> {
    cursor: usize,
    buffer: Vec<u8>,
    name: &'a str,
}

impl Deref for BufferPreallocated<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer[..self.cursor]
    }
}

impl<'a> BufferPreallocated<'a> {
    /// Create a buffer of fixed sized
    pub fn new(buffer_size: usize) -> Self {
        BufferPreallocated {
            cursor: 0,
            buffer: vec![0_u8; buffer_size],
            name: "",
        }
    }

    /// Define a name to buffer, mostly for debugging purpose
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
}

impl DerefMut for BufferPreallocated<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer[..self.cursor]
    }
}

impl Buffer for BufferPreallocated<'_> {
    fn append(
        &mut self,
        other: &[u8],
        evinceable: Option<usize>,
    ) -> Result<bool, StreamParserError> {
        tracing::trace!("[{}] Appending to buffer", self.name);

        let mut eviction = false;
        let free_space = self.buffer.len() - self.cursor;
        tracing::trace!("[{}] free space : {free_space}", self.name);
        tracing::trace!("[{}] other len : {}", self.name, other.len());
        // Attempt to append data size greater than the available space
        if other.len() > free_space {
            // Trying to evince useless data
            tracing::debug!(
                "[{}] Evinceable ? {} {:?}",
                self.name,
                evinceable.is_some(),
                evinceable
            );

            tracing::trace!("Before eviction {}", debug!(&self.buffer[..self.cursor]));

            match evinceable {
                Some(0) | None => {
                    return Err(StreamParserError::ExceededBuffer {
                        buffer_size: self.buffer.len(),
                        data_size: other.len(),
                    })
                }
                Some(evince_number) => {
                    tracing::debug!("[{}] Evincing data", self.name);
                    tracing::trace!("[{}] Evincing {} bytes", self.name, evince_number);
                    for (i, x) in (evince_number..self.cursor).enumerate() {
                        self.buffer[i] = self.buffer[x];
                    }
                    self.cursor -= evince_number;
                    eviction = true;
                }
            }
        }

        self.buffer[self.cursor..other.len() + self.cursor].clone_from_slice(other);
        self.cursor += other.len();
        tracing::trace!("After eviction {}", debug!(&self.buffer[..self.cursor]));
        Ok(eviction)
    }

    fn copy_from(&mut self, source: &Self, evinceable: Option<usize>) {
        tracing::trace!("[{}] Cloning from buffer", self.name);

        // Re-init existing data
        self.clear();
        self.append(source, evinceable).unwrap();
    }

    fn clear(&mut self) {
        tracing::trace!("[{}] Clearing buffer", self.name);
        self.cursor = 0;
    }

    fn incr_cursor(&mut self, offset: usize) {
        self.cursor += offset;
    }

    fn get_write_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn reset(&mut self) {
        self.cursor = 0
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::buffers::preallocated::BufferPreallocated;
    use crate::traits::Buffer;

    #[test]
    fn append_with_eviction() {
        let mut buffer = BufferPreallocated::new(6);
        let data = b"abc";
        buffer.append(data, None).unwrap();
        buffer.append(b"de", None).unwrap();
        buffer.append(b"123", Some(2)).unwrap();
        assert_eq!(&b"cde123", &buffer.deref());
    }
}
