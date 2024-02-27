use nom_stream_parser::{Heuristic, StartGroup};
use nom_stream_parser::buffers::preallocated::BufferPreallocated;
use nom_stream_parser::StreamParser;
use utils::parsers::{parse_data, start_group_parenthesis};
use utils::source::Source;

#[test_pretty_log::test]
fn test_stream_parser() {
    let data = b"(1,2,3,(4,5,6),7,8,9)(61,36,16,20,7)(62))(45,18,47,77,a,40,59,21)(21,6)<.(39,4,3)(76,47,83,55,33,5,10,20,28)R(2,63,67,40,57))(14,34)(";
    let expected = vec![
        vec![4, 5, 6],
        vec![61, 36, 16, 20, 7],
        vec![62],
        vec![21, 6],
        vec![39, 4, 3],
        vec![76, 47, 83, 55, 33, 5, 10, 20, 28],
        vec![2, 63, 67, 40, 57],
        vec![14, 34],
    ];
    let source = Source::new(data).with_chunk_size(20);
    let mut save_buffer = BufferPreallocated::new(40).with_name("save buffer");
    let mut work_buffer = BufferPreallocated::new(40).with_name("work buffer");
    let parser = parse_data;
    let group_start = StartGroup {
        parser: start_group_parenthesis,
        start_character: b"(",
    };
    let stream = StreamParser::new(
        source,
        &mut save_buffer,
        &mut work_buffer,
        parser,
        Heuristic::SearchGroup(group_start),
    );

    let result = stream.flatten().collect::<Vec<Vec<u8>>>();
    assert_eq!(expected, result);
}
