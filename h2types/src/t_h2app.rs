//! H2 Application Traits

use crate::H2ErrorOut;

use crate::{FrameFlags, FrameType, StreamId};

/// Implement to Provide H2
pub trait H2FrameReceiver {
    /// Provides the frame header
    fn header(
        &mut self,
        _frame_type: FrameType,
        _frame_flags: FrameFlags,
        _stream_id: StreamId,
        _payload_len: u32,
    ) -> Result<(), H2ErrorOut>;
    /// Provides the payload
    fn payload(&mut self, _payload: &[u8], _remaining: u32) -> Result<(), H2ErrorOut>;
}
