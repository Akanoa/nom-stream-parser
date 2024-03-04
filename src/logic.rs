use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Read;

use crate::errors::StreamParserError;
use crate::heuristic::Heuristic;
use crate::parser_state::{ParsableState, ParserState, SearchState};
use crate::traits::{Buffer, ParserFunction};
use crate::{debug, DataSource};

pub(crate) type Logic<St, R> = Box<dyn FnMut(&mut St) -> Option<Result<R, StreamParserError>>>;

/// The return state of a parsing iteration
#[derive(Debug)]
pub enum ReturnState<R> {
    /// The parser haven't enough data to take a decision
    NeedMoreData,
    /// The parser generated a new data
    Data(R),
    /// An error occurred in the iteration
    /// either buffer overflow or parsing error
    Error(StreamParserError),
}

pub(crate) fn iteration_logic<'a, I, B, R, O>() -> Logic<ParserState<'a, I, B, R, O>, O>
where
    I: Iterator<Item = &'a [u8]>,
    R: Read,
    B: Buffer,
    O: Debug,
{
    Box::new(|x: &mut ParserState<'a, I, B, R, O>| {
        tracing::info!("New next() call");
        tracing::debug!("At next() call state : {:?}", x.state);
        tracing::trace!("Cursor: {}", x.cursor);

        // Eviction de donnée

        loop {
            // x.i += 1;
            // if x.i > 20 {
            //     return None;
            // }

            // We yield more data from source iterator if the previous
            // ask for or if the work_buffer is empty
            let current_len = x.work_buffer[x.cursor..].len();
            if let ((_, ParsableState::NeedMoreData), _) | (_, 0) = (&x.state, current_len) {
                tracing::debug!("Asking for more data");

                let mut empty = false;

                match &mut x.data_source {
                    DataSource::Iterator(iterator) => {
                        let data = iterator.next();
                        if let Some(data) = data {
                            tracing::trace!("New data : {}", debug!(data));
                            let eviction = x.work_buffer.append(data, Some(x.cursor));
                            match eviction {
                                Err(err) => return Some(Err(err)),
                                Ok(true) => {
                                    x.cursor = 0;
                                }
                                _ => {}
                            };
                            // The work buffer can be parsed now
                            x.state.1 = ParsableState::MaybeParsable;
                        } else {
                            empty = true
                        }
                    }
                    DataSource::Reader(reader) => {
                        let size = reader.read(&mut x.work_buffer[x.cursor..]);

                        match size {
                            Err(err) => return Some(Err(err.into())),
                            Ok(0) => empty = true,
                            Ok(size) => x.work_buffer.incr_cursor(size),
                        }
                    }
                }

                if empty {
                    return None;
                }
            }

            let parse_internal_result = parse_internal(
                x.work_buffer,
                &mut x.state,
                &mut x.cursor,
                x.parser,
                &x.heuristic,
            );

            match parse_internal_result {
                Ok(Some(data)) => return Some(Ok(data)),
                Err(err) => return Some(Err(err)),
                _ => {}
            }
        }
    })
}

pub fn parse_internal<B: Buffer, R: Debug>(
    work_buffer: &mut B,
    state: &mut (SearchState, ParsableState),
    cursor: &mut usize,
    parser: ParserFunction<R>,
    heuristic: &RefCell<Heuristic>,
) -> Result<Option<R>, StreamParserError> {
    tracing::debug!("Parsing work buffer");

    let return_state = parsing_logic(work_buffer, state, cursor, parser, heuristic);

    match return_state {
        Ok(ReturnState::NeedMoreData) => {
            state.1 = ParsableState::NeedMoreData;
            Ok(None)
        }
        Ok(ReturnState::Data(data)) => {
            state.1 = ParsableState::MaybeParsable;
            Ok(Some(data))
        }
        Ok(ReturnState::Error(err)) => {
            tracing::debug!("Yield an error");
            Err(err)
        }
        Err(err) => Err(err),
    }
}

fn parsing_logic<B: Buffer, R: Debug>(
    work_buffer: &mut B,
    state: &mut (SearchState, ParsableState),
    cursor: &mut usize,
    parser: ParserFunction<R>,
    start_group: &RefCell<Heuristic>,
) -> Result<ReturnState<R>, StreamParserError>
where
{
    if let (SearchState::SearchForStart, _) = state {
        if let Some(return_state) =
            start_group
                .borrow_mut()
                .start_group(work_buffer, state, cursor)?
        {
            return Ok(return_state);
        }
    }

    let input = &work_buffer[*cursor..];

    tracing::debug!("Parsing data {}", debug!(input));

    let result_parse = parser(input);

    match result_parse {
        Ok((remain, data)) => {
            tracing::debug!("Successfully parse tokens");
            tracing::trace!("Found data = {data:?}");
            tracing::trace!("Remaining token = {}", debug!(remain));
            *cursor += input.len() - remain.len();
            tracing::trace!(
                "Shift cursor from {} to {}",
                debug!(input),
                debug!(&work_buffer[*cursor..])
            );
            state.0 = SearchState::SearchForStart;

            // if work_buffer[*cursor..].is_empty() {
            //     save_buffer.clear()
            // }

            return Ok(ReturnState::Data(data));
        }
        Err(nom::Err::Incomplete(_)) => {
            tracing::debug!("Not enough data to decide");
            tracing::trace!("In {}", debug!(input));
            tracing::debug!("Asking for more data on incomplete data");
            //save_buffer.clear();
            //save_buffer.append(input, Some(*cursor))?;
        }
        Err(err) => {
            tracing::trace!(
                "Error in {} to {}",
                debug!(input),
                debug!(&work_buffer[*cursor..])
            );

            state.0 = SearchState::SearchForStart;

            if input.is_empty() {
                return Ok(ReturnState::NeedMoreData);
            }

            let old_input = &work_buffer[*cursor..];

            *cursor += 1;

            return Ok(ReturnState::Error(err.map_input(|_| old_input).into()));
        }
    }
    Ok(ReturnState::NeedMoreData)
}
