//! H2 Frame builder traits

/// Implement to receive built frame/s
pub trait FrameReceiver {
    /// Receive the built frame
    fn frame(&mut self, frame: &[u8]);
}

use crate::StreamId;

/// Implement to provide required data for raw framing
pub trait RawFrameBuilder {
    /// Provide the StreamId to be used
    fn stream_id(&mut self) -> StreamId;
    /// Provide the Raw flasg to be used
    fn flags(&mut self) -> u32;
}

/// Implement to provide required data for Settings framing
pub trait SettingsFrameBuilder {
    /// Provide the StreamId to be used
    fn stream_id(&mut self) -> StreamId;
    /// Provide the Ack flag to be used
    fn ack_flag(&mut self) -> bool;
}
