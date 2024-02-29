use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use nom_stream_parser::{Buffer, Heuristic, StartGroup, StreamParser, StreamParserError};
use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use utils::parsers::{parse_data, start_group_parenthesis};
use utils::seeder::SeederConfig;
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
        work_buffer,
        parser,
        Heuristic::SearchGroup(search_group_heuristic),
    );
    let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    Ok(result)
}

#[test]
fn failed_seed_15987178197214890543() {
    let mut rng = ChaCha8Rng::seed_from_u64(15987178197214890543);

    let config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    let (data_to_parse, expected) = config.generate(&mut rng);

    let source = Source::new(&data_to_parse).with_chunk_size(4096);

    let mut save_buffer = BufferPreallocated::new(1_048_576).with_name("save buffer");
    let mut work_buffer = BufferPreallocated::new(1_048_576).with_name("work buffer");
    let result = parse(source, &mut save_buffer, &mut work_buffer);
    assert_eq!(Ok(expected), result);
}

#[test]
fn failed_seed_3386706919782654474() {
    let mut rng = ChaCha8Rng::seed_from_u64(3386706919782654474);

    let config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    let (data_to_parse, expected) = config.generate(&mut rng);

    let source = Source::new(&data_to_parse).with_chunk_size(4096);

    let mut save_buffer = BufferPreallocated::new(1_048_576).with_name("save buffer");
    let mut work_buffer = BufferPreallocated::new(1_048_576).with_name("work buffer");
    let result = parse(source, &mut save_buffer, &mut work_buffer);
    assert_eq!(Ok(expected), result);
}

#[test]
fn failed_seed_720586935495819268() {
    let mut rng = ChaCha8Rng::seed_from_u64(720586935495819268);

    let config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    let (data_to_parse, expected) = config.generate(&mut rng);

    let source = Source::new(&data_to_parse).with_chunk_size(4096);

    let mut save_buffer = BufferPreallocated::new(1_048_576).with_name("save buffer");
    let mut work_buffer = BufferPreallocated::new(1_048_576).with_name("work buffer");
    let result = parse(source, &mut save_buffer, &mut work_buffer);
    assert_eq!(Ok(expected), result);
}
