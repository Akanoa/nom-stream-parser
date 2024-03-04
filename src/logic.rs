use std::cell::RefCell;
use std::fmt::Debug;

use crate::debug;
use crate::errors::StreamParserError;
use crate::heuristic::Heuristic;
use crate::parser_state::{ParsableState, SearchState};
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

pub fn parse_internal<B: Buffer, R: Debug, H: Heuristic>(
    work_buffer: &mut B,
    state: &mut (SearchState, ParsableState),
    cursor: &mut usize,
    parser: ParserFunction<R>,
    heuristic: &RefCell<H>,
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

fn parsing_logic<B: Buffer, R: Debug, H: Heuristic>(
    work_buffer: &mut B,
    state: &mut (SearchState, ParsableState),
    cursor: &mut usize,
    parser: ParserFunction<R>,
    start_group: &RefCell<H>,
) -> Result<ReturnState<R>, StreamParserError>
where
{
    if let (SearchState::SearchForStart, _) = state {
        if let Some(return_state) = start_group.borrow_mut().apply(work_buffer, state, cursor)? {
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
