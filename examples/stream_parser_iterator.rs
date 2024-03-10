use nom::{AsBytes, character, IResult};
use nom::bytes::streaming::tag;
use nom::combinator::map_parser;
use nom::multi::separated_list1;
use nom::sequence::delimited;

use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::builder::StreamParserBuilder;
use utils::source::Source;

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

fn main() {
    by_structure();
    #[cfg(feature = "builder")]
    by_builder();
}

fn by_structure() {
    let data = b"noise(1,4,3,4)###(2,5)".as_bytes();
    // The work_buffer is used both to parse data and to accumulate partials.
    // It must be sized according to your parsed data
    let mut work_buffer = BufferPreallocated::new(20);
    let source = Source::new(data).with_chunk_size(4);
    let stream = nom_stream_parser::stream_parsers::sync_iterator::StreamParser::new(
        source,
        &mut work_buffer,
        parser,
        // This heuristic is quite simple, move by one character if the parser
        // failed at current position
        nom_stream_parser::heuristic::Increment,
    );

    println!("By structure");

    for x in stream {
        match x {
            Ok(data) => println!("Data : {:?}", data),
            Err(err) => println!("Error: {}", err),
        }
    }

    println!("-----------------------------");
}

#[cfg(feature = "builder")]
fn by_builder() {
    let data = b"noise(1,4,3,4)###(2,5)".as_bytes();
    // The work_buffer is used both to parse data and to accumulate partials.
    // It must be sized according to your parsed data
    let mut work_buffer = BufferPreallocated::new(20);
    // Source is a chunked iterator over slice
    let source = Source::new(data).with_chunk_size(4);
    // Create StreamParser with Increment heuristic
    // This heuristic is quite simple, move by one character if the parser
    // failed at current position
    let stream = StreamParserBuilder::default()
        .parser(parser)
        .work_buffer(&mut work_buffer)
        // Set the Iterator which be used as data source
        .iterator(source)
        // Build the StreamParser
        .build()
        // The builder can fail if field is missing
        .unwrap()
        // Get the stream from StreamParser
        .stream();

    println!("By builder");

    for x in stream {
        match x {
            Ok(data) => println!("Data : {:?}", data),
            Err(err) => println!("Error: {}", err),
        }
    }

    println!("-----------------------------");
}
