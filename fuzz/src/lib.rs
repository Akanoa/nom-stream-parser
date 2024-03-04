use nom_stream_parser::{
    Buffer, DataSource, Heuristic, StartGroup, StreamParser, StreamParserError,
};
use utils::parsers::{parse_data, start_group_parenthesis};
use utils::source::Source;

pub fn parse<B: Buffer>(
    source: Source,
    work_buffer: &mut B,
) -> Result<Vec<Vec<u8>>, StreamParserError> {
    let parser = parse_data;
    let search_group_heuristic = StartGroup {
        parser: start_group_parenthesis,
        start_character: b"(",
    };
    let stream = StreamParser::new(
        DataSource::<_, &[u8]>::Iterator(source),
        work_buffer,
        parser,
        Heuristic::SearchGroup(search_group_heuristic),
    );
    let result: Vec<Vec<u8>> = stream.filter_map(|x| x.ok()).collect();
    Ok(result)
}
