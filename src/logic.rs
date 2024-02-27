use std::cell::RefCell;
use std::fmt::Debug;

use crate::debug;
use crate::errors::StreamParserError;
use crate::parser_state::{ParsableState, ParserState, SearchState};
use crate::search_group::Heuristic;
use crate::traits::{Buffer, ParserFunction};

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


pub(crate) fn iteration_logic<'a, I, B, R>() -> Logic<ParserState<'a, I, B, R>, R>
    where
        I: Iterator<Item = &'a [u8]>,
        B: Buffer,
        R: Debug,
{
    Box::new(|x: &mut ParserState<'a, I, B, R>| {
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

                let data = x.iterator.next();

                if let Some(data) = data {
                    tracing::trace!("New data : {}", debug!(data));
                    let eviction = x.work_buffer.append(data, Some(x.cursor)).unwrap();
                    tracing::debug!("After append work buffer");
                    if eviction {
                        x.cursor = 0;
                    }
                    // The work buffer can be parsed now
                    x.state.1 = ParsableState::MaybeParsable;
                } else {
                    if x.save_buffer.is_empty() {
                        tracing::trace!("No more data can be yield and save buffer empty");
                        return None;
                    }

                    if x.heuristic.borrow_mut().prevent_infinite_loop(x.save_buffer) {
                        return None
                    }
                }
            }

            tracing::debug!("Parsing work buffer");

            let return_state = parse_internal(
                x.save_buffer,
                x.work_buffer,
                &mut x.state,
                &mut x.cursor,
                x.parser,
                &x.heuristic,
            )
                .unwrap();

            match return_state {
                ReturnState::NeedMoreData => x.state.1 = ParsableState::NeedMoreData,
                ReturnState::Data(data) => {
                    x.state.1 = ParsableState::MaybeParsable;
                    return Some(Ok(data));
                }
                ReturnState::Error(err) => {
                    tracing::debug!("Yield an error");
                    return Some(Err(err));
                }
            }
        }
    })
}


fn parse_internal<'b, B: Buffer, R: Debug>(
    save_buffer: &mut B,
    work_buffer: &'b mut B,
    state: &mut (SearchState, ParsableState),
    cursor: &mut usize,
    parser: ParserFunction<R>,
    start_group: &RefCell<Heuristic>,
) -> Result<ReturnState<R>, StreamParserError>
    where
{

    if let (SearchState::SearchForStart, _) = state {
        if let Some(return_state) = start_group.borrow_mut().start_group(save_buffer, work_buffer, state, cursor)? {
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

            if work_buffer[*cursor..].is_empty() {
                save_buffer.clear()
            }

            return Ok(ReturnState::Data(data));
        }
        Err(nom::Err::Incomplete(_)) => {
            tracing::debug!("Not enough data to decide");
            tracing::trace!("In {}", debug!(input));
            tracing::debug!("Asking for more data on incomplete data");
            save_buffer.clear();
            save_buffer.append(input, Some(*cursor))?;
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



            return Ok(ReturnState::Error(err.map_input(|_|old_input).into()));
        }
    }
    Ok(ReturnState::NeedMoreData)
}