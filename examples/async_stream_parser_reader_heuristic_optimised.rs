use std::pin::pin;

use futures_lite::StreamExt;
use nom::bytes::streaming::tag;
use nom::character::complete::digit1;
use nom::combinator::map_parser;
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::{character, AsBytes, IResult};

use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::builder::StreamParserBuilder;
use nom_stream_parser::StartGroupByParser;

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

#[tokio::main]
async fn main() {
    by_structure().await;
    #[cfg(feature = "builder")]
    by_builder().await;
    #[cfg(feature = "builder")]
    by_builder_after_iterator().await;
}

async fn by_structure() {
    let data = b"noise%$3%(1,4,3,4)###%$89%(2,5)".as_bytes();
    let mut work_buffer = BufferPreallocated::new(20);
    // This heuristic try to found the start_character and
    // apply the parser defined to detect complex start group
    let heuristic = StartGroupByParser {
        parser: start_group_complex,
        start_character: "%".as_bytes(),
    };
    let mut stream = pin!(nom_stream_parser::stream_parsers::async_reader::stream(
        &mut work_buffer,
        parser,
        heuristic,
        data,
    ));

    println!("By structure");

    while let Some(x) = stream.next().await {
        match x {
            Ok(data) => println!("Data : {:?}", data),
            Err(err) => println!("Error: {}", err),
        }
    }

    println!("-----------------------------");
}

#[cfg(feature = "builder")]
async fn by_builder() {
    let data = b"noise%$3%(1,4,3,4)###%$89%(2,5)".as_bytes();
    // The work_buffer is used both to parse data and to accumulate partials.
    // It must be sized according to your parsed data
    let mut work_buffer = BufferPreallocated::new(20);

    // This heuristic try to found the start_character and
    // apply the parser defined to detect complex start group
    let heuristic = StartGroupByParser {
        parser: start_group_complex,
        start_character: "%".as_bytes(),
    };

    // Set the heuristic at the definition of the builder
    let stream = StreamParserBuilder::with_heuristic(heuristic)
        // Declare the parser asynchronous
        .asynchronous()
        .parser(parser)
        .work_buffer(&mut work_buffer)
        // Set the Iterator which be used as data source
        .reader(data)
        // Build the StreamParser
        .build()
        // The builder can fail if field is missing
        .unwrap()
        // Get the stream from StreamParser
        .stream();

    println!("By builder");

    let mut stream = pin!(stream);

    while let Some(x) = stream.next().await {
        match x {
            Ok(data) => println!("Data : {:?}", data),
            Err(err) => println!("Error: {}", err),
        }
    }

    println!("-----------------------------");
}

#[cfg(feature = "builder")]
async fn by_builder_after_iterator() {
    let data = b"noise%$3%(1,4,3,4)###%$89%(2,5)".as_bytes();
    // The work_buffer is used both to parse data and to accumulate partials.
    // It must be sized according to your parsed data
    let mut work_buffer = BufferPreallocated::new(20);

    // This heuristic try to found the start_character and
    // apply the parser defined to detect complex start group
    let heuristic = StartGroupByParser {
        parser: start_group_complex,
        start_character: "%".as_bytes(),
    };

    // Set the heuristic at the definition of the builder
    let stream = StreamParserBuilder::with_heuristic(heuristic)
        .parser(parser)
        .work_buffer(&mut work_buffer)
        // Set the Iterator which be used as data source
        .reader(data)
        // Declare the parser asynchronous
        .asynchronous()
        // Build the StreamParser
        .build()
        // The builder can fail if field is missing
        .unwrap()
        // Get the stream from StreamParser
        .stream();

    println!("By builder asynchronous declared after iterator");

    let mut stream = pin!(stream);

    while let Some(x) = stream.next().await {
        match x {
            Ok(data) => println!("Data : {:?}", data),
            Err(err) => println!("Error: {}", err),
        }
    }

    println!("-----------------------------");
}
