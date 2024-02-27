use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bench_macros::generate_bench;
use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::{Buffer, Heuristic, StartGroup, StreamParser, StreamParserError};
use utils::parsers::{parse_data, start_group_parenthesis};
use utils::seeder::{source_data, SeederConfig};
use utils::source::Source;

pub fn parse<B: Buffer>(
    source: Source,
    save_buffer: &mut B,
    work_buffer: &mut B,
) -> Result<Vec<Vec<u8>>, StreamParserError> {
    let parser = parse_data;
    let search_group_heuristic = StartGroup {
        parser: start_group_parenthesis,
        start_character: b"(",
    };
    let stream = StreamParser::new(
        source,
        save_buffer,
        work_buffer,
        parser,
        Heuristic::SearchGroup(search_group_heuristic),
    );
    let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    Ok(result)
}

generate_bench!(
    name = big_data;
    config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    seed = 42;
    parser = parse;
    buffer = BufferPreallocated::new(1_048_576);
    chunk_sizes = 4096
);

generate_bench!(
    name = hell_data;
    config = SeederConfig::new(14000, 30, 2, 4, 4, 10000, false);
    seed = 42;
    parser = parse;
    buffer = BufferPreallocated::new(500_048_576);
    chunk_sizes = 4096
);

generate_bench!(
    name = small_data;
    config = SeederConfig::new(14, 30, 2, 4, 4, 10, false);
    seed = 42;
    parser = parse;
    buffer = BufferPreallocated::new(1_048_576);
    chunk_sizes = 4096
);

criterion_main!(small_data, big_data, hell_data);
