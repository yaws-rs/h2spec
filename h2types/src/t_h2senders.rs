//! H2 (Builder) Sender Types

use crate::FrameFlags;
use crate::FrameType;
use crate::StreamId;

/// Required Frame metadata to send frames
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct H2FrameMeta {
    frame_type: FrameType,
    frame_flags: FrameFlags,
    stream_id: StreamId,
    payload_len: u32,
}

impl H2FrameMeta {
    /// As bytes using the provided buffer
    pub fn as_bytes<'d>(&self, buf: &'d mut [u8]) -> Option<&'d mut [u8]> {
        if buf.len() < 9 {
            return None;
        }
        let size_b = self.payload_len.to_be_bytes();
        buf[0] = size_b[1];
        buf[1] = size_b[2];
        buf[2] = size_b[3];
        buf[3] = self.frame_type.as_u8();
        buf[4] = self.frame_flags.as_u8();
        let stream_id = self.stream_id().as_u32();
        let stream_id_b = stream_id.to_be_bytes();
        buf[5] = stream_id_b[0];
        buf[6] = stream_id_b[1];
        buf[7] = stream_id_b[2];
        buf[8] = stream_id_b[3];

        Some(&mut buf[0..9])
    }
    /// Construct new
    #[inline]
    pub fn new(frame_type: FrameType) -> Self {
        Self {
            frame_type,
            frame_flags: FrameFlags::default(),
            stream_id: StreamId::default(),
            payload_len: 0,
        }
    }
    /// Get Frame flags
    #[inline]
    pub fn flags(&self) -> FrameFlags {
        self.frame_flags
    }
    /// Set Frame flags
    #[inline]
    pub fn set_flags(&mut self, flags: FrameFlags) {
        self.frame_flags = flags;
    }
    /// Get Stream Id
    #[inline]
    pub fn stream_id(&self) -> StreamId {
        self.stream_id
    }
    #[inline]
    /// Set Stream Id
    pub fn set_stream_id(&mut self, stream_id: StreamId) {
        self.stream_id = stream_id;
    }
    /// Get Payload len
    pub fn payload_len(&self) -> u32 {
        self.payload_len
    }
    /// Set Payload len. Maximum is 0xFFFFFF.
    #[must_use]
    pub fn set_payload_len(&mut self, payload_len: u32) -> bool {
        if payload_len > 0xFFFFFF {
            return false;
        }
        self.payload_len = payload_len;
        true
    }
}

/// Implement to send H2 Frames
pub trait H2FrameSender {
    /// Provide the required Frame metadata
    fn send_start_meta(&mut self) -> Option<H2FrameMeta>;
    /// Provide the required Frame payload up to provided buffer
    fn send_payload(&mut self, _buf: &mut [u8]) -> usize;
}
