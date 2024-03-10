use std::fmt::Debug;

use async_stream::stream;
use futures_lite::Stream;

use crate::{debug, ParserFunction};
use crate::{Buffer, StreamParserError};
use crate::heuristic::Heuristic;
use crate::logic::parse_internal;
use crate::parser_state::ParsableState;
use crate::stream_parsers::ParserCommonFields;

pub fn stream<'a, B, I, H, O>(
    work_buffer: &'a mut B,
    parser: ParserFunction<O>,
    heuristic: H,
    mut iterator: I,
) -> impl Stream<Item = Result<O, StreamParserError>> + '_
where
    I: Iterator<Item = &'a [u8]> + 'a,
    B: Buffer,
    H: Heuristic + Unpin + 'a,
    O: Debug + 'a,
{
    let mut common_fields = ParserCommonFields::new(work_buffer, parser, heuristic);

    stream! {
    loop {
        // We yield more data from source iterator if the previous
        // ask for or if the work_buffer is empty
        let current_len = common_fields.work_buffer[common_fields.cursor..].len();
        if let ((_, ParsableState::NeedMoreData), _) | (_, 0) = (&common_fields.state, current_len) {
            tracing::debug!("Asking for more data");

            let data = iterator.next();
            if let Some(data) = data {
                tracing::trace!("New data : {}", debug!(data));
                let eviction = common_fields.work_buffer.append(data, Some(common_fields.cursor));
                match eviction {
                    Err(err) => yield Err(err),
                    Ok(true) => {
                        common_fields.cursor = 0;
                    }
                    _ => {}
                };
                // The work buffer can be parsed now
                common_fields.state.1 = ParsableState::MaybeParsable;
            } else {
                break;
            }
        }

        let parse_internal_result = parse_internal(
            common_fields.work_buffer,
            &mut common_fields.state,
            &mut common_fields.cursor,
            common_fields.parser,
            &mut common_fields.heuristic,
        );

        match parse_internal_result {
            Ok(Some(data)) => yield Ok(data),
            Err(err) => yield Err(err),
            _ => {}
        }
    };

    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use futures_lite::StreamExt;
    use nom::AsBytes;

    use utils::parsers::{parse_data, start_group_parenthesis};
    use utils::source::Source;

    use crate::buffers::preallocated::BufferPreallocated;
    use crate::StartGroupByParser;
    use crate::stream_parsers::async_iterator::stream;

    #[tokio::test]
    async fn test_stream_async() {
        let data = b"noise(1,4,3,4)###(2,5)".as_bytes();
        let source = Source::new(data).with_chunk_size(4);
        let mut work_buffer = BufferPreallocated::new(20);
        let parser = parse_data;
        let heuristic = StartGroupByParser {
            parser: start_group_parenthesis,
            start_character: b"(",
        };
        let stream = stream(&mut work_buffer, parser, heuristic, source);
        let mut stream = pin!(stream);

        while let Some(x) = stream.next().await {
            match x {
                Ok(data) => println!("Data : {:?}", data),
                Err(err) => println!("Error: {}", err),
            }
        }
    }
}
