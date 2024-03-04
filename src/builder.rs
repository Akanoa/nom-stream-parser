use std::fmt::Debug;
use std::io::Read;

use derive_builder::Builder;

use crate::heuristic::{Heuristic, Increment};
use crate::{Buffer, ParserFunction};

#[derive(Builder)]
#[builder(pattern = "owned")]
#[builder(build_fn(skip))]
pub struct StreamParser<'a, B: Buffer, O: Debug, H: Heuristic = Increment> {
    #[allow(unused)]
    work_buffer: &'a mut B,
    #[allow(unused)]
    parser: ParserFunction<O>,
    #[allow(unused)]
    heuristic: H,
}

impl<'a, B: Buffer, O: Debug, H: Heuristic> StreamParserBuilder<'a, B, O, H> {
    pub fn reader<R: Read>(self, reader: R) -> StreamParserReaderBuilder<'a, B, R, O, H> {
        StreamParserReaderBuilder {
            reader: Some(reader),
            work_buffer: self.work_buffer,
            parser: self.parser,
            heuristic: self.heuristic,
        }
    }

    pub fn iterator<I: Iterator<Item = &'a [u8]>>(
        self,
        iterator: I,
    ) -> StreamParserIteratorBuilder<'a, B, I, O, H> {
        StreamParserIteratorBuilder {
            iterator: Some(iterator),
            work_buffer: self.work_buffer,
            parser: self.parser,
            heuristic: self.heuristic,
        }
    }
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct StreamParserIterator<'a, B, I, O, H = Increment>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    pub iterator: I,
    pub work_buffer: &'a mut B,
    pub parser: ParserFunction<O>,
    pub heuristic: H,
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct StreamParserReader<'a, B, R, O, H = Increment>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    pub reader: R,
    pub work_buffer: &'a mut B,
    pub parser: ParserFunction<O>,
    pub heuristic: H,
}

impl<'a, B, I, O, H> StreamParserIterator<'a, B, I, O, H>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    pub fn stream(self) -> crate::stream_parsers::sync_iterator::StreamParser<'a, I, B, O, H> {
        crate::stream_parsers::sync_iterator::StreamParser::new(
            self.iterator,
            self.work_buffer,
            self.parser,
            self.heuristic,
        )
    }
}

impl<'a, B, R, O, H> StreamParserReader<'a, B, R, O, H>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    pub fn stream(self) -> crate::stream_parsers::sync_reader::StreamParser<'a, R, B, O, H> {
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

    use utils::parsers::start_group_parenthesis;
    use utils::source::Source;

    use crate::buffers::preallocated::BufferPreallocated;
    use crate::builder::StreamParserBuilder;
    use crate::{Buffer, StartGroupByParser};

    #[test]
    fn test_builder() {
        {
            let data = b"noise(1,4,3,4)###(2,5)".as_bytes();
            let it = Source::new(data).with_chunk_size(4);
            let mut work_buffer = BufferPreallocated::new(20);
            let heuristic = StartGroupByParser {
                parser: start_group_parenthesis,
                start_character: b"(",
            };
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

            work_buffer.clear();

            println!("----------------------------------------");
            let heuristic = StartGroupByParser {
                parser: start_group_parenthesis,
                start_character: b"(",
            };

            let stream = StreamParserBuilder::default()
                .work_buffer(&mut work_buffer)
                .parser(parser)
                .reader(data)
                .heuristic(heuristic)
                .build()
                .unwrap()
                .stream();

            for x in stream {
                match x {
                    Ok(data) => println!("Data : {:?}", data),
                    Err(err) => println!("Error: {}", err),
                }
            }
        }
    }
}
