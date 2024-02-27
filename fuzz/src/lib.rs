use nom_stream_parser::{Buffer, Heuristic, StartGroup, StreamParser, StreamParserError};
use utils::parsers::{parse_data, start_group_parenthesis};
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
