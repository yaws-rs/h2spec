//! H2 Application Traits

use crate::H2ErrorOut;

use crate::{FrameFlags, FrameType, StreamId};

/// Reset H2 App
pub trait H2Reset {
    /// Implement to reset
    fn reset(&mut self);
}

/// Implement to Provide H2
pub trait H2FrameReceiver {
    fn header(
        &mut self,
        _frame_type: FrameType,
        _frame_flags: FrameFlags,
        _stream_id: StreamId,
        _payload_len: u32,
    ) -> Result<(), H2ErrorOut>;
    fn payload(&mut self, _payload: &[u8], _remaining: u32) -> Result<(), H2ErrorOut>;
}
