use std::fmt::Debug;
use std::io::Read;

use itertools::{unfold, Unfold};

use crate::heuristic::Heuristic;
use crate::logic::parse_internal;
use crate::parser_state::{ParsableState, SearchState};
use crate::stream_parsers::ParserCommonFields;
use crate::{Buffer, ParserFunction, StreamParserError};

type SteamUnfold<'a, R, B, O, H> =
    Unfold<ParserState<'a, R, B, O, H>, Logic<ParserState<'a, R, B, O, H>, O>>;

type Logic<St, O> = Box<dyn FnMut(&mut St) -> Option<Result<O, StreamParserError>>>;

struct ParserState<'a, R, B, O, H>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
{
    /// Iterated data
    pub reader: R,
    /// Buffer used when data must be accumulated
    pub common: ParserCommonFields<'a, B, O, H>,
}

impl<'a, R, B, O, H> ParserState<'a, R, B, O, H>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
{
    fn new(work_buffer: &'a mut B, reader: R, parser: ParserFunction<O>, heuristic: H) -> Self {
        Self {
            reader,
            common: ParserCommonFields {
                work_buffer,
                state: (SearchState::SearchForStart, ParsableState::NeedMoreData),
                cursor: 0,
                parser,
                heuristic,
                i: 0,
            },
        }
    }
}

pub struct StreamParser<'a, R, B, O, H>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    stream: SteamUnfold<'a, R, B, O, H>,
}

impl<'a, R, B, O, H> StreamParser<'a, R, B, O, H>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    pub fn new(reader: R, work_buffer: &'a mut B, parser: ParserFunction<O>, heuristic: H) -> Self {
        let logic_state = ParserState::new(work_buffer, reader, parser, heuristic);

        let stream = unfold(logic_state, iteration_logic());
        StreamParser { stream }
    }
}

impl<'a, R, B, O, H> Iterator for StreamParser<'a, R, B, O, H>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    type Item = Result<O, StreamParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next()
    }
}

fn iteration_logic<'a, R, B, O, H>() -> crate::logic::Logic<ParserState<'a, R, B, O, H>, O>
where
    R: Read,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    Box::new(|x: &mut ParserState<'a, R, B, O, H>| {
        tracing::info!("New next() call");
        tracing::debug!("At next() call state : {:?}", x.common.state);
        tracing::trace!("Cursor: {}", x.common.cursor);

        // Eviction de donnée

        loop {
            // x.common.i += 1;
            // if x.common.i > 1500 {
            //     return None;
            // }

            // We yield more data from source iterator if the previous
            // ask for or if the work_buffer is empty
            let full = x.common.work_buffer.len() - x.common.cursor;
            if let ((_, ParsableState::NeedMoreData), _) | (_, 0) = (&x.common.state, full) {
                tracing::trace!("Current cursor {}", x.common.cursor);
                if x.common.cursor != 0 {
                    x.common
                        .work_buffer
                        .evince(Some(x.common.cursor), "".as_bytes())
                        .unwrap();
                }

                tracing::debug!("Asking for more data");
                tracing::trace!(
                    "Work buffer len {}",
                    &x.common.work_buffer.get_write_buffer().len()
                );

                let size = x.reader.read(x.common.work_buffer.get_write_buffer());

                tracing::trace!("Read size {size:?}");

                match size {
                    Err(err) => return Some(Err(err.into())),
                    Ok(0) if x.common.work_buffer.get_write_buffer().is_empty() => {
                        return Some(Err(StreamParserError::ExceededBufferUnknownSize {
                            buffer_size: x.common.work_buffer.len(),
                        }))
                    }
                    Ok(0) => return None,
                    Ok(size) => {
                        x.common.state.1 = ParsableState::MaybeParsable;
                        x.common.work_buffer.incr_cursor(size);
                        x.common.cursor = 0;
                    }
                }
            }

            let parse_internal_result = parse_internal(
                x.common.work_buffer,
                &mut x.common.state,
                &mut x.common.cursor,
                x.common.parser,
                &mut x.common.heuristic,
            );

            match parse_internal_result {
                Ok(Some(data)) => return Some(Ok(data)),
                Err(err) => {
                    tracing::debug!("An error occured : {err}");
                    tracing::debug!("Cursor: {} ", x.common.cursor);
                    //x.common.work_buffer.reset();
                    return Some(Err(err));
                }
                _ => {
                    tracing::debug!("Resetting work buffer");
                    //x.common.work_buffer.reset()
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use nom::AsBytes;

    use utils::parsers::{parse_data, start_group_parenthesis};

    use crate::buffers::preallocated::BufferPreallocated;
    use crate::{StartGroupByParser, StreamParserError};

    use super::StreamParser;

    #[test_pretty_log::test]
    fn test_parse_with_reader() {
        let data = b"noise(1,5,3,4)###(2,5)(1,88,56,42,78,5)".as_bytes();
        let mut work_buffer = BufferPreallocated::new(20).with_name("work buffer");

        let heuristic = StartGroupByParser {
            parser: start_group_parenthesis,
            start_character: b"(",
        };

        let stream = StreamParser::new(data, &mut work_buffer, parse_data, heuristic);

        for x in stream {
            match x {
                Ok(data) => println!("Data : {:?}", data),
                Err(error @ StreamParserError::ExceededBufferUnknownSize { .. }) => {
                    eprintln!("Unrecoverable Error {}", error);
                    break;
                }
                Err(err) => println!("Error: {}", err),
            }
        }
    }
}
