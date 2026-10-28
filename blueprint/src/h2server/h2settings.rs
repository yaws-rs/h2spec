//! H2 Settings

use h2types::BuilderError;
use h2types::StreamId;
use h2types::{FrameReceiver, SettingsFrameBuilder};

#[derive(Default)]
pub(crate) enum SettingsFrameOut {
    #[default]
    Init,
    Bytes([u8; 60], usize),
    Errored(BuilderError),
}

impl SettingsFrameOut {
    pub(crate) fn frame_as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b, l) => Some(&b[..*l as usize]),
            _ => None,
        }
    }
}

impl SettingsFrameBuilder for SettingsFrameOut {
    fn stream_id(&mut self) -> StreamId {
        StreamId::unchecked_new(0)
    }
    fn ack_flag(&mut self) -> bool {
        false
    }
}

impl FrameReceiver for SettingsFrameOut {
    fn frame(&mut self, frame: &[u8]) {
        if frame.len() > 60 {
            *self = Self::Errored(BuilderError::Overflow);
        }
        let mut b = [0; 60];
        b[..frame.len()].copy_from_slice(frame);
        *self = Self::Bytes(b, frame.len());
    }
}
