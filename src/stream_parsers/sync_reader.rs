use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;

use itertools::{unfold, Unfold};

use crate::logic::parse_internal;
use crate::parser_state::{ParsableState, SearchState};
use crate::stream_parsers::ParserCommonFields;
use crate::{debug, Buffer, Heuristic, ParserFunction, StreamParserError};

type SteamUnfold<'a, R, B, O> =
    Unfold<ParserState<'a, R, B, O>, Logic<ParserState<'a, R, B, O>, O>>;

type Logic<St, O> = Box<dyn FnMut(&mut St) -> Option<Result<O, StreamParserError>>>;

struct ParserState<'a, R, B, O>
where
    R: Read,
    B: Buffer,
{
    /// Iterated data
    pub reader: R,
    /// Buffer used when data must be accumulated
    pub common: ParserCommonFields<'a, B, O>,
}

impl<'a, R, B, O> ParserState<'a, R, B, O>
where
    R: Read,
    B: Buffer,
{
    fn new(
        work_buffer: &'a mut B,
        reader: R,
        parser: ParserFunction<O>,
        start_group: Heuristic<'a>,
    ) -> Self {
        Self {
            reader,
            common: ParserCommonFields {
                work_buffer,
                state: (SearchState::SearchForStart, ParsableState::NeedMoreData),
                cursor: 0,
                parser,
                heuristic: RefCell::new(start_group),
                i: 0,
            },
        }
    }
}

pub struct StreamParser<'a, R, B, O>
where
    R: Read,
    B: Buffer,
    O: Debug,
{
    stream: SteamUnfold<'a, R, B, O>,
}

impl<'a, R, B, O> StreamParser<'a, R, B, O>
where
    R: Read,
    B: Buffer,
    O: Debug,
{
    pub fn new(
        reader: R,
        work_buffer: &'a mut B,
        parser: ParserFunction<O>,
        heuristic: Heuristic<'a>,
    ) -> Self {
        let logic_state = ParserState::new(work_buffer, reader, parser, heuristic);

        let stream = unfold(logic_state, iteration_logic());
        StreamParser { stream }
    }
}

impl<'a, R, B, O> Iterator for StreamParser<'a, R, B, O>
where
    R: Read,
    B: Buffer,
    O: Debug,
{
    type Item = Result<O, StreamParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next()
    }
}

fn iteration_logic<'a, R, B, O>() -> crate::logic::Logic<ParserState<'a, R, B, O>, O>
where
    R: Read,
    B: Buffer,
    O: Debug,
{
    Box::new(|x: &mut ParserState<'a, R, B, O>| {
        tracing::info!("New next() call");
        tracing::debug!("At next() call state : {:?}", x.common.state);
        tracing::trace!("Cursor: {}", x.common.cursor);

        // Eviction de donnée

        loop {
            // x.i += 1;
            // if x.i > 20 {
            //     return None;
            // }

            // We yield more data from source iterator if the previous
            // ask for or if the work_buffer is empty
            let full = x.common.work_buffer.len() - x.common.cursor;
            if let ((_, ParsableState::NeedMoreData), _) | (_, 0) = (&x.common.state, full) {
                x.common.work_buffer.reset();
                x.common.cursor = 0;
                tracing::debug!("Asking for more data");

                tracing::trace!(
                    "Work buffer content {}",
                    debug!(&x.common.work_buffer.get_write_buffer())
                );

                tracing::trace!(
                    "Work buffer len {}",
                    &x.common.work_buffer.get_write_buffer().len()
                );

                let size = x.reader.read(x.common.work_buffer.get_write_buffer());

                tracing::trace!("Read size {size:?}");

                match size {
                    Err(err) => return Some(Err(err.into())),
                    Ok(0) => return None,
                    Ok(size) => {
                        x.common.state.1 = ParsableState::MaybeParsable;
                        x.common.work_buffer.incr_cursor(size)
                    }
                }
            }

            let parse_internal_result = parse_internal(
                x.common.work_buffer,
                &mut x.common.state,
                &mut x.common.cursor,
                x.common.parser,
                &x.common.heuristic,
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
    use crate::{Heuristic, StartGroup};

    use super::StreamParser;

    #[test]
    fn test_parse_with_reader() {
        let data = b"noise(1,a,3,4)###(2,5)".as_bytes();
        //let it = Source::new(data).with_chunk_size(2);
        //let it = vec![b"noise(1,2,3,4)".as_bytes()].into_iter();
        let mut work_buffer = BufferPreallocated::new(10).with_name("work buffer");
        let heuristic = Heuristic::SearchGroup(StartGroup {
            parser: start_group_parenthesis,
            start_character: b"(",
        });

        let stream = StreamParser::new(data, &mut work_buffer, parse_data, Heuristic::Increment);

        for x in stream {
            match x {
                Ok(data) => println!("Data : {:?}", data),
                Err(err) => println!("Error: {}", err),
            }
        }
    }
}
