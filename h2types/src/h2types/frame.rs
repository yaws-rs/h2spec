//! Frame types

use crate::StreamId;

/// Frame Type RFC s. 6
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FrameType {
    /// s. 6.1. DATA (0x00)
    Data,
    /// s. 6.2. HEADERS (0x01)
    Headers,
    /// s. 6.3. PRIORITY (0x02) Deprecated by RFC 9113
    Priority,
    /// s. 6.4. RST_STREAM (0x03)
    RstStream,
    /// s. 6.5. SETTINGS (0x04)
    Settings,
    /// s. 6.6. PUSH_PROMISE (0x05)
    PushPromise,
    /// s. 6.7. PING (0x06)
    Ping,
    /// s. 6.8. GOAWAY (0x07)
    GoAway,
    /// s. 6.9. WINDOW_UPDATE (0x08)
    WindowUpdate,
    /// s. 6.10. CONTINUATION (0x09)
    Continuation,
    /// Unknown type of frame
    Unknown(u8),
}

impl FrameType {
    /// Construct from raw byte
    pub fn from_u8(i: u8) -> Self {
        match i {
            0 => Self::Data,
            1 => Self::Headers,
            2 => Self::Priority,
            3 => Self::RstStream,
            4 => Self::Settings,
            5 => Self::PushPromise,
            6 => Self::Ping,
            7 => Self::GoAway,
            8 => Self::WindowUpdate,
            9 => Self::Continuation,
            _ => Self::Unknown(i),
        }
    }
    /// As byte
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Data => 0,
            Self::Headers => 1,
            Self::Priority => 2,
            Self::RstStream => 3,
            Self::Settings => 4,
            Self::PushPromise => 5,
            Self::Ping => 6,
            Self::GoAway => 7,
            Self::WindowUpdate => 8,
            Self::Continuation => 9,
            Self::Unknown(i) => *i,
        }
    }
}

/// Frame Flags
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct FrameFlags(u8);

impl FrameFlags {
    /// Construct from u8
    pub fn from_u8(i: u8) -> Self {
        Self(i)
    }
    /// As byte
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}
