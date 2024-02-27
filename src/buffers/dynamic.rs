use std::ops::Deref;

use crate::errors::StreamParserError;
use crate::traits::Buffer;

#[derive(Default)]
pub struct BufferDynamic {
    buffer: Vec<u8>,
}

impl Deref for BufferDynamic {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl BufferDynamic {
    pub fn new() -> Self {
        BufferDynamic::default()
    }
}

impl Buffer for BufferDynamic {
    fn append(
        &mut self,
        other: &[u8],
        _evinceable: Option<usize>,
    ) -> Result<bool, StreamParserError> {
        self.buffer.extend_from_slice(other);

        Ok(false)
    }

    fn copy_from(&mut self, source: &Self, evinceable: Option<usize>) {
        tracing::trace!("Cloning from buffer");

        // Re-init existing data
        self.clear();
        self.append(source, evinceable).unwrap();
    }

    fn clear(&mut self) {
        tracing::trace!("Clearing from buffer");
        self.buffer.clear()
    }
}
