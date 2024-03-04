use std::cell::RefCell;
use std::fmt::Debug;

use itertools::{unfold, Unfold};

use crate::heuristic::Heuristic;
use crate::logic::parse_internal;
use crate::parser_state::{ParsableState, SearchState};
use crate::stream_parsers::ParserCommonFields;
use crate::{debug, Buffer, ParserFunction, StreamParserError};

type SteamUnfold<'a, I, B, O, H> =
    Unfold<ParserState<'a, I, B, O, H>, Logic<ParserState<'a, I, B, O, H>, O>>;

type Logic<St, O> = Box<dyn FnMut(&mut St) -> Option<Result<O, StreamParserError>>>;

struct ParserState<'a, I, B, O, H>
where
    I: Iterator<Item = &'a [u8]>,
    H: Heuristic,
    B: Buffer,
{
    /// Iterated data
    pub iterator: I,
    /// Buffer used when data must be accumulated
    pub common: ParserCommonFields<'a, B, O, H>,
}

impl<'a, I, B, O, H> ParserState<'a, I, B, O, H>
where
    I: Iterator<Item = &'a [u8]>,
    H: Heuristic,
    B: Buffer,
{
    fn new(work_buffer: &'a mut B, iterator: I, parser: ParserFunction<O>, heuristic: H) -> Self {
        Self {
            iterator,
            common: ParserCommonFields {
                work_buffer,
                state: (SearchState::SearchForStart, ParsableState::NeedMoreData),
                cursor: 0,
                parser,
                heuristic: RefCell::new(heuristic),
                i: 0,
            },
        }
    }
}

pub struct StreamParser<'a, I, B, O, H>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    stream: SteamUnfold<'a, I, B, O, H>,
}

impl<'a, I, B, O, H> StreamParser<'a, I, B, O, H>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    pub fn new(
        iterator: I,
        work_buffer: &'a mut B,
        parser: ParserFunction<O>,
        heuristic: H,
    ) -> Self {
        let logic_state = ParserState::new(work_buffer, iterator, parser, heuristic);

        let stream = unfold(logic_state, iteration_logic());
        StreamParser { stream }
    }
}

impl<'a, I, B, O, H> Iterator for StreamParser<'a, I, B, O, H>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    H: Heuristic,
    O: Debug,
{
    type Item = Result<O, StreamParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next()
    }
}

fn iteration_logic<'a, I, B, O, H>() -> crate::logic::Logic<ParserState<'a, I, B, O, H>, O>
where
    I: Iterator<Item = &'a [u8]>,
    H: Heuristic,
    B: Buffer,
    O: Debug,
{
    Box::new(|x: &mut ParserState<'a, I, B, O, H>| {
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
            let current_len = x.common.work_buffer[x.common.cursor..].len();
            if let ((_, ParsableState::NeedMoreData), _) | (_, 0) = (&x.common.state, current_len) {
                tracing::debug!("Asking for more data");

                let data = x.iterator.next();
                if let Some(data) = data {
                    tracing::trace!("New data : {}", debug!(data));
                    let eviction = x.common.work_buffer.append(data, Some(x.common.cursor));
                    match eviction {
                        Err(err) => return Some(Err(err)),
                        Ok(true) => {
                            x.common.cursor = 0;
                        }
                        _ => {}
                    };
                    // The work buffer can be parsed now
                    x.common.state.1 = ParsableState::MaybeParsable;
                } else {
                    return None;
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
                Err(err) => return Some(Err(err)),
                _ => {}
            }
        }
    })
}
