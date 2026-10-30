//! H2 Parser

mod p_h2headers;
pub use p_h2headers::*;

use crate::FrameParserReceiver;
use crate::ParsingError;
use crate::StreamId;
use crate::{FrameFlags, FrameType};

/// H2 Frame Parser
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameParser {
    status: ParsingStatus,
    cur_frame_id: usize,
    cur_body_len: u32,
}

/// Discard the amount parsed from input buffer
pub type Discard = usize;

/// Remaining payload in the H2 Frame
pub type Remaining = u32;

/// What the H2 Frame parser currently expects
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Expect {
    /// Header
    #[default]
    Header,
    /// Payload
    Payload(Remaining),
}

/// H2 Frame Parsing Status
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum ParsingStatus {
    /// Parsing is initialized but not run
    #[default]
    Initialized,
    /// Parsing is incomplete at Expect with Discard bytes
    Incomplete(Expect, Discard),
    /// Parsing is complete at Expect with Discard bytes
    Complete(Expect, Discard),
}

impl FrameParser {
    /// Create new parser with initialized state
    pub fn new() -> Self {
        Self::default()
    }
    /// Parse the body from the next N bytes up to payload length.
    /// The parser must be in the correct state to expect payload of remaining length.
    pub fn take_body<R: FrameParserReceiver>(
        &mut self,
        r: &mut R,
        input: &[u8],
    ) -> Result<ParsingStatus, ParsingError> {
        let remaining = match self.status {
            ParsingStatus::Incomplete(Expect::Payload(remaining), _) => remaining,
            _ => return Err(ParsingError::ExpectedStatePayload),
        };
        if remaining == 0 {
            return Ok(ParsingStatus::Complete(Expect::Payload(0), 0));
        }
        if input.len() == 0 {
            return Ok(ParsingStatus::Incomplete(Expect::Payload(remaining), 0));
        }

        let payload = if remaining as usize > input.len() {
            &input
        } else {
            &input[..remaining as usize]
        };

        let aft_remaining = remaining - payload.len() as u32;
        r.payload(self.cur_frame_id, &payload, aft_remaining);

        let status = match aft_remaining {
            0 => ParsingStatus::Complete(Expect::Payload(0), payload.len()),
            _ => ParsingStatus::Incomplete(Expect::Payload(aft_remaining), payload.len()),
        };

        Ok(status)
    }
    /// Parse the header from the next 9 bytes
    pub fn take_header<R: FrameParserReceiver>(
        &mut self,
        r: &mut R,
        input: &[u8],
    ) -> Result<ParsingStatus, ParsingError> {
        if input.len() < 9 {
            return Ok(ParsingStatus::Incomplete(Expect::Header, 0));
        }
        let len_u32 = u32::from_be_bytes([0, input[0], input[1], input[2]]);
        let frame_type = FrameType::from_u8(input[3]);
        let frame_flags = FrameFlags::from_u8(input[4]);
        let stream_u32 = u32::from_be_bytes([input[5], input[6], input[7], input[8]]);
        let stream_id = match StreamId::new(stream_u32) {
            Some(sid) => sid,
            None => return Err(ParsingError::InvalidStreamId),
        };

        let frame_id = r.header(frame_type, frame_flags, stream_id, len_u32);

        self.cur_frame_id = frame_id;

        self.cur_body_len = len_u32;

        return Ok(ParsingStatus::Complete(Expect::Header, 9));
    }
    /// Start or continue parsing frames
    pub fn parse<R: FrameParserReceiver>(
        &mut self,
        r: &mut R,
        input: &[u8],
    ) -> Result<ParsingStatus, ParsingError> {
        let mut discard = 0;
        loop {
            let new_status = match self.status {
                ParsingStatus::Initialized
                | ParsingStatus::Incomplete(Expect::Header, _)
                | ParsingStatus::Complete(Expect::Payload(0), _) => {
                    self.take_header(r, &input[discard..])?
                }
                ParsingStatus::Incomplete(Expect::Payload(p), _) if p > 0 => {
                    let status = self.take_body(r, &input[discard..])?;
                    status
                }
                _ => {
                    return Err(ParsingError::InvalidState);
                }
            };
            match new_status {
                ParsingStatus::Initialized => return Err(ParsingError::InvalidState),
                ParsingStatus::Complete(Expect::Header, n_discard) => {
                    discard += n_discard;
                    if self.cur_body_len == 0 {
                        self.status = ParsingStatus::Complete(Expect::Payload(0), discard);
                        if input.len() - discard > 0 {
                            continue;
                        } else {
                            break;
                        }
                    }
                    self.status =
                        ParsingStatus::Incomplete(Expect::Payload(self.cur_body_len), discard);
                    continue;
                }
                ParsingStatus::Complete(Expect::Payload(_), n_discard) => {
                    discard += n_discard;
                    let remaining = input.len() - discard;
                    if remaining == 0 {
                        self.cur_body_len = 0;
                        self.status = ParsingStatus::Complete(Expect::Payload(0), discard);
                        break;
                    } else {
                        self.status = ParsingStatus::Incomplete(Expect::Header, discard);
                        continue;
                    }
                }
                ParsingStatus::Incomplete(expect, n_discard) => {
                    discard += n_discard;
                    self.status = ParsingStatus::Incomplete(expect, discard);
                    break;
                }
            }
        }

        Ok(self.status)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use rstest::*;

    #[derive(Debug, Default)]
    struct TestReceiver {
        headers: Vec<String>,
        payload: Vec<String>,
        cur_frame: usize,
    }

    impl FrameParserReceiver for TestReceiver {
        fn header(
            &mut self,
            frame_type: FrameType,
            frame_flags: FrameFlags,
            stream_id: StreamId,
            payload_len: u32,
        ) -> usize {
            self.cur_frame += 1;
            self.headers.push(format!(
                "F[{}] FrameType/{:?}, FrameFlags/{:?}, StreamId/{:?}, PayloadLen/{}",
                self.cur_frame, frame_type, frame_flags, stream_id, payload_len
            ));
            self.cur_frame
        }
        fn payload(&mut self, frame_id: usize, payload: &[u8], remaining: u32) {
            self.payload.push(format!(
                "F[{}] Payload[{}]/{:?} Remaining/{}",
                frame_id,
                payload.len(),
                payload,
                remaining
            ));
        }
    }

    #[derive(Debug)]
    #[allow(dead_code)] // analysis ignores insta's debug assert
    struct Tc {
        input: &'static [u8],
        res: Result<ParsingStatus, ParsingError>,
        parser: FrameParser,
        receiver: TestReceiver,
    }

    pub(crate) fn ctx_insta(ctx: Context) -> String {
        let case_id = ctx.case.unwrap().to_string();
        format!("{}-{}", case_id, ctx.name)
    }

    #[rstest]
    #[case(&[0, 0, 18, 4, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 100, 0, 4, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 4, 8, 0, 0, 0, 0, 0, 62, 127, 0, 1, 0, 0, 49, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 83, 69, 84, 84, 73, 78, 71, 83, 58, 32, 115, 101, 114, 118, 101, 114, 32, 97, 116, 116, 101, 109, 112, 116, 101, 100, 32, 116, 111, 32, 101, 110, 97, 98, 108, 101, 32, 112, 117, 115, 104])]
    fn h2frame_parse_complete(#[context] ctx: Context, #[case] input: &'static [u8]) {
        let mut parser = FrameParser::new();
        let mut receiver = TestReceiver::default();
        let res = parser.parse(&mut receiver, input);

        assert_eq!(
            Ok(ParsingStatus::Complete(Expect::Payload(0), input.len())),
            res
        );
        insta::assert_debug_snapshot!(
            ctx_insta(ctx),
            Tc {
                input,
                res,
                parser,
                receiver
            }
        );
    }

    #[rstest]
    #[case(&[0, 0, 18, 4, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 100, 0, 4, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 4, 8, 0, 0, 0, 0, 0, 62, 127, 0, 1, 0, 0, 49, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 83, 69, 84, 84, 73, 78, 71, 83, 58, 32, 115, 101, 114, 118, 101, 114, 32, 97, 116, 116, 101, 109, 112, 116, 101, 100, 32, 116, 111, 32, 101, 110, 97, 98, 108, 101, 32, 112, 117, 115], 1)]
    fn h2frame_parse_incomplete_payload(
        #[context] ctx: Context,
        #[case] input: &'static [u8],
        #[case] missing: usize,
    ) {
        let mut parser = FrameParser::new();
        let mut receiver = TestReceiver::default();
        let res = parser.parse(&mut receiver, input);

        assert_eq!(
            Ok(ParsingStatus::Incomplete(
                Expect::Payload(missing as u32),
                input.len()
            )),
            res
        );
        insta::assert_debug_snapshot!(
            ctx_insta(ctx),
            Tc {
                input,
                res,
                parser,
                receiver
            }
        );
    }

    #[rstest]
    #[case(&[0, 0, 18, 4, 0, 0])]
    fn h2frame_parse_incomplete_header(#[context] ctx: Context, #[case] input: &'static [u8]) {
        let mut parser = FrameParser::new();
        let mut receiver = TestReceiver::default();
        let res = parser.parse(&mut receiver, input);

        assert_eq!(Ok(ParsingStatus::Incomplete(Expect::Header, 0)), res);
        insta::assert_debug_snapshot!(
            ctx_insta(ctx),
            Tc {
                input,
                res,
                parser,
                receiver
            }
        );
    }

    #[rstest]
    #[case(&[0, 0, 18, 4, 0, 0], &[0, 0, 0], 18)]
    fn h2frame_parse_2step_header(
        #[case] input: &'static [u8],
        #[case] input2: &'static [u8],
        #[case] expected_payload_len: usize,
    ) {
        let mut parser = FrameParser::new();
        let mut receiver = TestReceiver::default();
        let res1 = parser.parse(&mut receiver, input);
        assert_eq!(Ok(ParsingStatus::Incomplete(Expect::Header, 0)), res1);

        let mut combined = Vec::with_capacity(input.len() + input2.len());
        combined.extend(input);
        combined.extend(input2);

        let res2 = parser.parse(&mut receiver, &combined);
        assert_eq!(
            Ok(ParsingStatus::Incomplete(
                Expect::Payload(expected_payload_len as u32),
                9
            )),
            res2
        );

        let payload = vec![0; expected_payload_len];
        let res3 = parser.parse(&mut receiver, &payload);
        assert_eq!(
            Ok(ParsingStatus::Complete(
                Expect::Payload(0),
                expected_payload_len
            )),
            res3
        );

        let res4 = parser.parse(&mut receiver, &[]);
        assert_eq!(Ok(ParsingStatus::Incomplete(Expect::Header, 0)), res4);
    }

    #[rstest]
    #[case(&[0, 0, 0, 4, 0, 0, 0, 0, 0])]
    fn h2frame_parse_zero_payload(#[context] ctx: Context, #[case] input: &'static [u8]) {
        let mut parser = FrameParser::new();
        let mut receiver = TestReceiver::default();
        let res = parser.parse(&mut receiver, input);

        assert_eq!(
            Ok(ParsingStatus::Complete(Expect::Payload(0), input.len())),
            res
        );
        insta::assert_debug_snapshot!(
            ctx_insta(ctx),
            Tc {
                input,
                res,
                parser,
                receiver
            }
        );
    }

    #[rstest]
    #[case(&[0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 2, 4, 0, 0, 0, 0, 0])]
    fn h2frame_parse_zero_payload_cont(#[context] ctx: Context, #[case] input: &'static [u8]) {
        let mut parser = FrameParser::new();
        let mut receiver = TestReceiver::default();
        let res = parser.parse(&mut receiver, input);

        assert_eq!(
            Ok(ParsingStatus::Incomplete(Expect::Payload(2), input.len())),
            res
        );
        insta::assert_debug_snapshot!(
            ctx_insta(ctx),
            Tc {
                input,
                res,
                parser,
                receiver
            }
        );
    }
}
