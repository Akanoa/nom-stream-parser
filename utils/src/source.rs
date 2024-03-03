#[derive(Clone)]
pub struct Source<'a> {
    data: &'a [u8],
    cursor: usize,
    chunk_size: usize,
}

impl Source<'_> {
    /// Get inner data len
    pub fn get_len(&self) -> usize {
        self.data.len()
    }
}

impl<'a> Iterator for Source<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor + self.chunk_size > self.data.len() {
            let final_data = &self.data[self.cursor..];
            self.cursor += final_data.len();

            return if final_data.is_empty() {
                None
            } else {
                Some(final_data)
            };
        }

        let next_data = Some(&self.data[self.cursor..self.cursor + self.chunk_size]);
        self.cursor += self.chunk_size;
        next_data
    }
}

impl<'a> Source<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            cursor: 0,
            chunk_size: 4,
        }
    }
    pub fn with_chunk_size(self, chunk_size: usize) -> Self {
        Self {
            data: self.data,
            chunk_size,
            cursor: self.cursor,
        }
    }
}
