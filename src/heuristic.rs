use nom::Parser;

use crate::debug;
use crate::errors::StreamParserError;
use crate::logic::ReturnState;
use crate::parser_state::{ParsableState, SearchState};
use crate::traits::{Buffer, ParserFunctionStartGroup};

/// The variant heuristics which define the
/// behavior to found the next relevant
pub enum Heuristic<'a> {
    /// Move cursor by one position
    Increment,
    /// Search for group
    SearchGroup(StartGroup<'a>),
}

impl<'a> Heuristic<'a> {
    pub(crate) fn start_group<'b, B: Buffer, R>(
        &'b mut self,
        work_buffer: &'b mut B,
        state: &mut (SearchState, ParsableState),
        cursor: &mut usize,
    ) -> Result<Option<ReturnState<R>>, StreamParserError> {
        match self {
            Heuristic::Increment => Ok(None),
            Heuristic::SearchGroup(start_group) => {
                search_for_start_group_from_parser(work_buffer, state, cursor, start_group)
            }
        }
    }
}

/// Data structure used by [Heuristic::SearchGroup]
pub struct StartGroup<'a> {
    /// The parser which define whether the
    /// cursor reaches a start group
    pub parser: ParserFunctionStartGroup,
    /// The first byte of a start group
    pub start_character: &'a [u8],
}

fn search_for_start_group_from_parser<B: Buffer, R>(
    work_buffer: &mut B,
    state: &mut (SearchState, ParsableState),
    cursor: &mut usize,
    start_group: &mut StartGroup,
) -> Result<Option<ReturnState<R>>, StreamParserError> {
    let input = &work_buffer[*cursor..];
    tracing::debug!("Search for a new group start");
    tracing::trace!("In {}", debug!(input));
    // On vérifie si le début de groupe est dans le buffer mémoire
    let result_search_for_start =
        nom::bytes::complete::take_until::<_, _, ()>(start_group.start_character)(input);

    match result_search_for_start {
        Err(_) => {
            tracing::debug!("No group start found");
            tracing::trace!("In {}", debug!(input));
            tracing::debug!("Cleaning buffers");
            //save_buffer.clear();
            work_buffer.clear();
            *cursor = 0;
            tracing::trace!("Asking for more data");
            state.0 = SearchState::SearchForStart;
            return Ok(Some(ReturnState::NeedMoreData));
        }
        Ok((remain, garbage)) => {
            tracing::debug!("Found group start");
            tracing::trace!(
                "In {} remain = {} garbage = {}",
                debug!(input),
                debug!(remain),
                debug!(garbage)
            );

            let result_group_start_complete = start_group.parser.parse(remain);

            match result_group_start_complete {
                Ok((_remain, garbage2)) => {
                    tracing::debug!("Found complete group start");
                    tracing::trace!(
                        "In {} remain = {} garbage2 = {}",
                        debug!(input),
                        debug!(remain),
                        debug!(garbage2)
                    );
                    *cursor += garbage.len() + garbage2.len();
                    state.0 = SearchState::StartFound
                }
                Err(nom::Err::Incomplete(_)) => {
                    return Ok(Some(ReturnState::NeedMoreData));
                }
                Err(_) => {
                    tracing::debug!("No group start found");
                    tracing::trace!("In {}", debug!(input));
                    tracing::debug!("Cleaning buffers");
                    //save_buffer.clear();
                    work_buffer.clear();
                    *cursor = 0;
                    tracing::trace!("Asking for more data");
                    state.0 = SearchState::SearchForStart;
                    return Ok(Some(ReturnState::NeedMoreData));
                }
            }
        }
    }

    Ok(None)
}
