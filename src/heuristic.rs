use nom::Parser;

use crate::debug;
use crate::errors::StreamParserError;
use crate::logic::ReturnState;
use crate::parser_state::{ParsableState, SearchState};
use crate::traits::{Buffer, ParserFunctionStartGroup};

pub trait Heuristic {
    fn apply<B: Buffer, R>(
        &mut self,
        work_buffer: &mut B,
        state: &mut (SearchState, ParsableState),
        cursor: &mut usize,
    ) -> Result<Option<ReturnState<R>>, StreamParserError>;
}

pub struct Increment;

impl Heuristic for Increment {
    fn apply<B: Buffer, R>(
        &mut self,
        _work_buffer: &mut B,
        _state: &mut (SearchState, ParsableState),
        _cursor: &mut usize,
    ) -> Result<Option<ReturnState<R>>, StreamParserError> {
        Ok(None)
    }
}

/// Data structure used by [EnumHeuristic::SearchGroup]
pub struct StartGroupByParser<'a> {
    /// The parser which define whether the
    /// cursor reaches a start group
    pub parser: ParserFunctionStartGroup,
    /// The first byte of a start group
    pub start_character: &'a [u8],
}

impl<'a> Heuristic for StartGroupByParser<'a> {
    fn apply<B: Buffer, R>(
        &mut self,
        work_buffer: &mut B,
        state: &mut (SearchState, ParsableState),
        cursor: &mut usize,
    ) -> Result<Option<ReturnState<R>>, StreamParserError> {
        let input = &work_buffer[*cursor..];
        tracing::debug!("Search for a new group start");
        tracing::trace!("In {}", debug!(input));
        // On vérifie si le début de groupe est dans le buffer mémoire
        let result_search_for_start =
            nom::bytes::complete::take_until::<_, _, ()>(self.start_character)(input);

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

                let result_group_start_complete = self.parser.parse(remain);

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
}
