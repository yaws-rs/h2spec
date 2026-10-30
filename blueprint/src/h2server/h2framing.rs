//! H2 Framing

use super::H2Serving;
use super::StageHandlerStatus;
use h2types::FrameParserReceiver;
use h2types::H2ErrorOut;
use h2types::H2FrameReceiver;
use h2types::{FrameFlags, FrameType, StreamId};
use h2types::{FrameParser, ParsingStatus};

struct FramingRelay<'r, App> {
    h2serving: &'r mut H2Serving<App>,
    cur_parser_frame: usize,
    error_out: Option<H2ErrorOut>,
}

impl<'r, App> FramingRelay<'r, App> {
    fn with_h2serving(h2s: &'r mut H2Serving<App>) -> Self {
        FramingRelay {
            h2serving: h2s,
            cur_parser_frame: 0,
            error_out: None,
        }
    }
}

impl<'r, App> FrameParserReceiver for FramingRelay<'r, App>
where
    App: H2FrameReceiver,
{
    #[inline]
    fn header(
        &mut self,
        frame_type: FrameType,
        frame_flags: FrameFlags,
        stream_id: StreamId,
        payload_len: u32,
    ) -> usize {
        self.cur_parser_frame += 1;
        match self
            .h2serving
            .app
            .header(frame_type, frame_flags, stream_id, payload_len)
        {
            Ok(_) => (),
            Err(e) => {
                self.error_out = Some(e);
            }
        };
        self.cur_parser_frame
    }
    #[inline]
    fn payload(&mut self, _frame_id: usize, payload: &[u8], remaining: u32) {
        match self.h2serving.app.payload(payload, remaining) {
            Ok(_) => (),
            Err(e) => {
                self.error_out = Some(e);
            }
        }
    }
}

impl<App> H2Serving<App>
where
    App: H2FrameReceiver,
{
    pub(crate) fn handle_h2frames(&mut self, left_in_b: &[u8]) -> StageHandlerStatus {
        let mut parser = FrameParser::default();
        let mut relay = FramingRelay::with_h2serving(self);

        match parser.parse(&mut relay, left_in_b) {
            Ok(ParsingStatus::Complete(_, discard) | ParsingStatus::Incomplete(_, discard)) => {
                StageHandlerStatus::with_ok(discard)
            }
            Err(_) => StageHandlerStatus::with_error_out(relay.error_out.into(), 0),
            Ok(ParsingStatus::Initialized) => StageHandlerStatus::with_ok(0),
        }
    }
}
