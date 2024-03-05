use nom::{AsBytes, character, IResult};
use nom::bytes::streaming::tag;
use nom::character::complete::digit1;
use nom::combinator::map_parser;
use nom::multi::separated_list1;
use nom::sequence::delimited;

use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::builder::StreamParserBuilder;
use nom_stream_parser::StartGroupByParser;
use utils::source::Source;

pub fn start_group_complex(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (_, data) = delimited(tag("%$"), digit1, tag("%("))(input)?;
    let next = 2 + data.len() + 1;
    Ok((&input[next..], &input[..next]))
}

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
    let data = b"noise%$3%(1,4,3,4)###%$89%(2,5)".as_bytes();
    let mut work_buffer = BufferPreallocated::new(20);
    // Source is a chunked iterator over slice
    let source = Source::new(data).with_chunk_size(4);
    // This heuristic try to found the start_character and
    // apply the parser defined to detect complex start group
    let heuristic = StartGroupByParser {
        parser: start_group_complex,
        start_character: "%".as_bytes(),
    };
    let stream = nom_stream_parser::stream_parsers::sync_iterator::StreamParser::new(
        source,
        &mut work_buffer,
        parser,
        heuristic,
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
    let data = b"noise%$3%(1,4,3,4)###%$89%(2,5)".as_bytes();
    // The work_buffer is used both to parse data and to accumulate partials.
    // It must be sized according to your parsed data
    let mut work_buffer = BufferPreallocated::new(20);
    // Source is a chunked iterator over slice
    let source = Source::new(data).with_chunk_size(8);

    // This heuristic try to found the start_character and
    // apply the parser defined to detect complex start group
    let heuristic = StartGroupByParser {
        parser: start_group_complex,
        start_character: "%".as_bytes(),
    };

    let stream = StreamParserBuilder::default()
        .parser(parser)
        // Set the heuristic
        .heuristic(heuristic)
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
