use std::fmt::Debug;
use std::io::Read;

use async_stream::stream;
use futures_lite::Stream;

use crate::heuristic::Heuristic;
use crate::logic::parse_internal;
use crate::parser_state::ParsableState;
use crate::stream_parsers::ParserCommonFields;
use crate::ParserFunction;
use crate::{Buffer, StreamParserError};

pub fn stream<'a, B, R, H, O>(
    work_buffer: &'a mut B,
    parser: ParserFunction<O>,
    heuristic: H,
    mut reader: R,
) -> impl Stream<Item = Result<O, StreamParserError>> + '_
where
    R: Read + 'a,
    B: Buffer,
    H: Heuristic + Unpin + 'a,
    O: Debug + 'a,
{
    let mut common_fields = ParserCommonFields::new(work_buffer, parser, heuristic);

    stream! {
        loop {
            // x.common.i += 1;
            // if x.common.i > 1500 {
            //     return None;
            // }

            // We yield more data from source iterator if the previous
            // ask for or if the work_buffer is empty
            let full = common_fields.work_buffer.len() - common_fields.cursor;
            if let ((_, ParsableState::NeedMoreData), _) | (_, 0) = (&common_fields.state, full) {
                tracing::trace!("Current cursor {}", common_fields.cursor);
                if common_fields.cursor != 0 {
                    common_fields
                        .work_buffer
                        .evince(Some(common_fields.cursor), "".as_bytes())
                        .unwrap();
                }

                tracing::debug!("Asking for more data");
                tracing::trace!(
                    "Work buffer len {}",
                    &common_fields.work_buffer.get_write_buffer().len()
                );

                let size = reader.read(common_fields.work_buffer.get_write_buffer());

                tracing::trace!("Read size {size:?}");

                match size {
                    Err(err) => yield Err(err.into()),
                    Ok(0) if common_fields.work_buffer.get_write_buffer().is_empty() => {
                        yield Err(StreamParserError::ExceededBufferUnknownSize {
                            buffer_size: common_fields.work_buffer.len(),
                        })
                    }
                    Ok(0) => break,
                    Ok(size) => {
                        common_fields.state.1 = ParsableState::MaybeParsable;
                        common_fields.work_buffer.incr_cursor(size);
                        common_fields.cursor = 0;
                    }
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
                Err(err) => {
                    tracing::debug!("An error occured : {err}");
                    tracing::debug!("Cursor: {} ", common_fields.cursor);
                    //common_fields.work_buffer.reset();
                    yield Err(err);
                }
                _ => {
                    tracing::debug!("Resetting work buffer");
                    //common_fields.work_buffer.reset()
                }
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
    use crate::stream_parsers::async_iterator::stream;
    use crate::StartGroupByParser;

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
