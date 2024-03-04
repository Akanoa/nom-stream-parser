#![no_main]

use libfuzzer_sys::fuzz_target;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::builder::StreamParserBuilder;
use nom_stream_parser::StartGroupByParser;
use utils::parsers::{parse_data, start_group_parenthesis};
use utils::seeder::SeederConfig;
use utils::source::Source;

fuzz_target!(|seed: u64| {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    let (data_to_parse, expected) = config.generate(&mut rng);

    let source = Source::new(&data_to_parse).with_chunk_size(4096);
    let mut work_buffer = BufferPreallocated::new(1_048_576).with_name("work buffer");

    let heuristic = StartGroupByParser {
        parser: start_group_parenthesis,
        start_character: b"(",
    };

    let stream = StreamParserBuilder::default()
        .work_buffer(&mut work_buffer)
        .parser(parse_data)
        .iterator(source)
        .heuristic(heuristic)
        .build()
        .unwrap()
        .stream();

    let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    assert_eq!(expected, result);
});
