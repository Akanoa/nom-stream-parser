use std::fmt::Debug;
use std::io::Read;

use derive_builder::Builder;

use crate::{Buffer, Heuristic, ParserFunction};

#[derive(Builder)]
#[builder(pattern = "owned")]
#[builder(name = "StreamParserBuilder")]
struct StreamParserBuilderInternal<'a, B: Buffer, O: Debug> {
    work_buffer: &'a mut B,
    parser: ParserFunction<O>,
    #[builder(default = "Heuristic::Increment")]
    heuristic: Heuristic<'a>,
}

impl<'a, B: Buffer, O: Debug> StreamParserBuilder<'a, B, O> {
    pub fn reader<R: Read>(self, reader: R) -> StreamParserBuilderReader<'a, B, R, O> {
        StreamParserBuilderReader {
            reader: Some(reader),
            work_buffer: self.work_buffer,
            parser: self.parser,
            heuristic: self.heuristic,
        }
    }

    pub fn iterator<I: Iterator<Item = &'a [u8]>>(
        self,
        iterator: I,
    ) -> StreamParserBuilderIterator<'a, B, I, O> {
        StreamParserBuilderIterator {
            iterator: Some(iterator),
            work_buffer: self.work_buffer,
            parser: self.parser,
            heuristic: self.heuristic,
        }
    }
}

#[derive(Builder)]
#[builder(pattern = "owned")]
#[builder(name = "StreamParserBuilderIterator")]
pub struct StreamParserBuilderInternalIterator<'a, B, I, O>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    O: Debug,
{
    pub iterator: I,
    pub work_buffer: &'a mut B,
    pub parser: ParserFunction<O>,
    #[builder(default = "Heuristic::Increment")]
    pub heuristic: Heuristic<'a>,
}

#[derive(Builder)]
#[builder(pattern = "owned")]
#[builder(name = "StreamParserBuilderReader")]
pub struct StreamParserBuilderInternalReader<'a, B, R, O>
where
    R: Read,
    B: Buffer,
    O: Debug,
{
    pub reader: R,
    pub work_buffer: &'a mut B,
    pub parser: ParserFunction<O>,
    #[builder(default = "Heuristic::Increment")]
    pub heuristic: Heuristic<'a>,
}

impl<'a, B, I, O> StreamParserBuilderInternalIterator<'a, B, I, O>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    O: Debug,
{
    pub fn stream(self) -> crate::stream_parsers::sync_iterator::StreamParser<'a, I, B, O> {
        crate::stream_parsers::sync_iterator::StreamParser::new(
            self.iterator,
            self.work_buffer,
            self.parser,
            self.heuristic,
        )
    }
}

impl<'a, B, R, O> StreamParserBuilderInternalReader<'a, B, R, O>
where
    R: Read,
    B: Buffer,
    O: Debug,
{
    pub fn stream(self) -> crate::stream_parsers::sync_reader::StreamParser<'a, R, B, O> {
        crate::stream_parsers::sync_reader::StreamParser::new(
            self.reader,
            self.work_buffer,
            self.parser,
            self.heuristic,
        )
    }
}

#[cfg(test)]
mod tests {
    use nom::bytes::streaming::tag;
    use nom::combinator::map_parser;
    use nom::multi::separated_list1;
    use nom::sequence::delimited;
    use nom::{character, AsBytes, IResult};

    use crate::buffers::preallocated::BufferPreallocated;
    use crate::builder::StreamParserBuilder;
    use crate::{Heuristic, StartGroup};

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

    fn start_group_parenthesis(input: &[u8]) -> IResult<&[u8], &[u8]> {
        nom::bytes::streaming::take_until("(")(input)
    }

    #[test]
    fn test_builder() {
        {
            let data = b"noise(1,a,3,4)###(2,5)".as_bytes();
            let it = Source::new(data).with_chunk_size(2);
            //let it = vec![b"noise(1,2,3,4)".as_bytes()].into_iter();
            let mut work_buffer = BufferPreallocated::new(20);
            let heuristic = Heuristic::SearchGroup(StartGroup {
                parser: start_group_parenthesis,
                start_character: b"(",
            });
            fn parser(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
                delimited(
                    tag("("),
                    separated_list1(
                        tag(","),
                        map_parser(character::streaming::digit1, character::complete::u8),
                    ),
                    tag(")"),
                )(input)
            }
            let stream = StreamParserBuilder::default()
                .work_buffer(&mut work_buffer)
                .parser(parser)
                .heuristic(heuristic)
                .iterator(it)
                .build()
                .unwrap()
                .stream();

            for x in stream {
                match x {
                    Ok(data) => println!("Data : {:?}", data),
                    Err(err) => println!("Error: {}", err),
                }
            }

            let stream = StreamParserBuilder::default()
                .work_buffer(&mut work_buffer)
                .parser(parser)
                .reader(data)
                .build()
                .unwrap()
                .stream();
        }
    }
}
