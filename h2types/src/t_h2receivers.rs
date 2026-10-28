//! H2 (Parser) Receiver Types

use crate::{FrameFlags, FrameType, StreamId};

/// Implement to recieve the parsed H2 Frame
pub trait FrameParserReceiver {
    /// Provides the header of the H2 Frame. Return an identifier to identify subsequent payload.
    fn header(&mut self, _: FrameType, _: FrameFlags, _: StreamId, _payload_len: u32) -> usize;
    /// Provides part or full payload of the H2 Frame.
    fn payload(&mut self, _frame_id: usize, _payload: &[u8], _remaining_len: u32);
}
