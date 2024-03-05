use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bench_macros::generate_bench_iterator;
use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::{Buffer, StartGroupByParser, StreamParserError};
use utils::parsers::{parse_data, start_group_parenthesis};
use utils::seeder::{source_data, SeederConfig};
use utils::source::Source;

pub fn parse<B: Buffer>(
    source: Source,
    work_buffer: &mut B,
) -> Result<Vec<Vec<u8>>, StreamParserError> {
    let parser = parse_data;
    let search_group_heuristic = StartGroupByParser {
        parser: start_group_parenthesis,
        start_character: b"(",
    };
    let stream = nom_stream_parser::stream_parsers::sync_iterator::StreamParser::new(
        source,
        work_buffer,
        parser,
        search_group_heuristic,
    );
    let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    Ok(result)
}

generate_bench_iterator!(
    name = big_data;
    config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    seed = 42;
    parser = parse;
    buffer = BufferPreallocated::new(1_048_576);
    chunk_sizes = 4096
);

generate_bench_iterator!(
    name = hell_data;
    config = SeederConfig::new(14000, 30, 2, 4, 4, 10000, false);
    seed = 42;
    parser = parse;
    buffer = BufferPreallocated::new(500_048_576);
    chunk_sizes = 4096
);

generate_bench_iterator!(
    name = small_data;
    config = SeederConfig::new(14, 30, 2, 4, 4, 10, false);
    seed = 42;
    parser = parse;
    buffer = BufferPreallocated::new(1_048_576);
    chunk_sizes = 4096
);

criterion_main!(small_data, big_data, hell_data);
