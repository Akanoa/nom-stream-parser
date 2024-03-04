use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::builder::StreamParserBuilder;
use nom_stream_parser::{debug, Heuristic, StartGroup, StreamParserError};
use utils::parsers::{parse_data, start_group_parenthesis};
use utils::seeder::SeederConfig;

#[test_pretty_log::test]
fn failed_seed_585186476717703168() {
    let mut rng = ChaCha8Rng::seed_from_u64(585186476717703168);

    let config = SeederConfig::new(140, 30, 2, 4, 4, 10, false);
    let (data_to_parse, expected) = config.generate(&mut rng);

    dbg!(data_to_parse.len());
    dbg!(debug!(&data_to_parse));

    //let source = Source::new(&data_to_parse).with_chunk_size(4096);
    let mut work_buffer = BufferPreallocated::new(1_048_576).with_name("work buffer");

    let heuristic = Heuristic::SearchGroup(StartGroup {
        parser: start_group_parenthesis,
        start_character: b"(",
    });

    let stream = StreamParserBuilder::default()
        .work_buffer(&mut work_buffer)
        .parser(parse_data)
        .reader(data_to_parse.as_slice())
        .heuristic(heuristic)
        .build()
        .unwrap()
        .stream();

    let mut result = vec![];

    for x in stream {
        match x {
            Ok(data) => {
                println!("Data : {:?}", data);
                result.push(data)
            }
            Err(error @ StreamParserError::ExceededBufferUnknownSize { .. }) => {
                eprintln!("Unrecoverable Error {}", error);
                break;
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    // let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    assert_eq!(expected, result);
}

#[test_pretty_log::test]
fn failed_seed_585186476717703168x10() {
    let mut rng = ChaCha8Rng::seed_from_u64(585186476717703168);

    let config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    let (data_to_parse, expected) = config.generate(&mut rng);

    dbg!(data_to_parse.len());
    dbg!(debug!(&data_to_parse));

    //let source = Source::new(&data_to_parse).with_chunk_size(4096);
    let mut work_buffer = BufferPreallocated::new(1_048_576).with_name("work buffer");

    let heuristic = Heuristic::SearchGroup(StartGroup {
        parser: start_group_parenthesis,
        start_character: b"(",
    });

    let stream = StreamParserBuilder::default()
        .work_buffer(&mut work_buffer)
        .parser(parse_data)
        .reader(data_to_parse.as_slice())
        .heuristic(heuristic)
        .build()
        .unwrap()
        .stream();

    let mut result = vec![];

    for x in stream {
        match x {
            Ok(data) => {
                //println!("Data : {:?}", data);
                result.push(data)
            }
            Err(error @ StreamParserError::ExceededBufferUnknownSize { .. }) => {
                eprintln!("Unrecoverable Error {}", error);
                break;
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    dbg!(result.len());
    dbg!(expected.len());

    // let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    assert_eq!(expected, result);
}

#[test]
fn failed_seed_42949672960() {
    let mut rng = ChaCha8Rng::seed_from_u64(42949672960);

    let config = SeederConfig::new(1400, 30, 2, 4, 4, 1000, false);
    let (data_to_parse, expected) = config.generate(&mut rng);

    let mut work_buffer = BufferPreallocated::new(1_048_576).with_name("work buffer");

    let heuristic = Heuristic::SearchGroup(StartGroup {
        parser: start_group_parenthesis,
        start_character: b"(",
    });

    let stream = StreamParserBuilder::default()
        .work_buffer(&mut work_buffer)
        .parser(parse_data)
        .reader(data_to_parse.as_slice())
        .heuristic(heuristic)
        .build()
        .unwrap()
        .stream();

    let mut result = vec![];

    for x in stream {
        match x {
            Ok(data) => {
                //println!("Data : {:?}", data);
                result.push(data)
            }
            Err(error @ StreamParserError::ExceededBufferUnknownSize { .. }) => {
                eprintln!("Unrecoverable Error {}", error);
                break;
            }
            Err(err) => {
                //println!("Error: {}", err)
            }
        }
    }

    dbg!(result.len());
    dbg!(expected.len());

    // let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    assert_eq!(expected, result);
}
