//! h2 error/s

/// RFC 9113 s.7 canonical H2 Errors
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum H2ErrorOut {
    /// **NO_ERROR (0x00)**
    ///
    /// ##
    /// The associated condition is not a result of an error. For example, a GOAWAY might include this code to indicate graceful shutdown of a connection.
    No,
    /// **PROTOCOL_ERROR (0x01)**
    ///
    /// ##
    /// The endpoint detected an unspecific protocol error. This error is for use when a more specific error code is not available.
    Protocol,
    /// **INTERNAL_ERROR (0x02)**
    ///
    /// ##
    /// The endpoint encountered an unexpected internal error.
    Internal,
    /// **FLOW_CONTROL_ERROR (0x03)**
    ///
    /// ##
    /// The endpoint detected that its peer violated the flow-control protocol.
    FlowControl,
    /// **SETTINGS_TIMEOUT (0x04)**
    ///
    /// ##
    /// The endpoint sent a SETTINGS frame but did not receive a response in a timely manner. See Section 6.5.3 ("Settings Synchronization").
    SettingsTimeout,
    /// **STREAM_CLOSED (0x05)**
    ///
    /// ##
    /// The endpoint received a frame with an invalid size.
    StreamClosed,
    /// **FRAME_SIZE_ERROR (0x06)**
    ///
    /// ##
    /// The endpoint received a frame with an invalid size.
    FrameSize,
    /// **REFUSED_STREAM (0x07)**
    ///
    /// ##
    /// The endpoint refused the stream prior to performing any application processing (see Section 8.7 for details).
    RefusedStream,
    /// **CANCEL (0x08)**
    ///
    /// ##
    /// The endpoint uses this error code to indicate that the stream is no longer needed.
    Cancel,
    /// **COMPRESSION_ERROR (0x09)**
    ///
    /// ##
    /// The endpoint is unable to maintain the field section compression context for the connection.
    Compression,
    /// **CONNECT_ERROR (0x0a)**
    ///
    /// ##
    /// The connection established in response to a CONNECT request (Section 8.5) was reset or abnormally closed.
    Connect,
    /// **ENHANCE_YOUR_CALM (0x0b)**
    ///
    /// ##
    /// The endpoint detected that its peer is exhibiting a behavior that might be generating excessive load.
    EnhanceYourCalm,
    /// **INADEQUATE_SECURITY (0x0c)**
    ///
    /// ##
    /// The underlying transport has properties that do not meet minimum security requirements (see Section 9.2).
    InadequateSecurity,
    /// **HTTP_1_1_REQUIRED (0x0d)**
    ///
    /// ##
    /// The endpoint requires that HTTP/1.1 be used instead of HTTP/2.
    Http11Required,
    /// Unknown to impl / non-RFC Error must be treated as Internal Error
    Unknown(u8),
}

impl From<Option<H2ErrorOut>> for H2ErrorOut {
    #[inline]
    fn from(oe: Option<H2ErrorOut>) -> H2ErrorOut {
        match oe {
            Some(e) => e,
            None => H2ErrorOut::Internal,
        }
    }
}

impl H2ErrorOut {
    /// As a byte
    #[inline]
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::No => 0x00,
            Self::Protocol => 0x01,
            Self::Internal => 0x02,
            Self::FlowControl => 0x03,
            Self::SettingsTimeout => 0x04,
            Self::StreamClosed => 0x05,
            Self::FrameSize => 0x06,
            Self::RefusedStream => 0x07,
            Self::Cancel => 0x08,
            Self::Compression => 0x09,
            Self::Connect => 0x0a,
            Self::EnhanceYourCalm => 0x0b,
            Self::InadequateSecurity => 0x0c,
            Self::Http11Required => 0x0d,
            Self::Unknown(u) => *u,
        }
    }
    /// Construct from a byte
    #[inline]
    pub fn from_u8(u: u8) -> Self {
        match u {
            0x00 => Self::No,
            0x01 => Self::Protocol,
            0x02 => Self::Internal,
            0x03 => Self::FlowControl,
            0x04 => Self::SettingsTimeout,
            0x05 => Self::StreamClosed,
            0x06 => Self::FrameSize,
            0x07 => Self::RefusedStream,
            0x08 => Self::Cancel,
            0x09 => Self::Compression,
            0x0a => Self::Connect,
            0x0b => Self::EnhanceYourCalm,
            0x0c => Self::InadequateSecurity,
            0x0d => Self::Http11Required,
            _ => Self::Unknown(u),
        }
    }
}

/// H2 Errors
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum H2Error {
    /// Settings related error, typically due to RFC 9113 s. 6.5.2 rules.
    Settings(H2SettingsError),
}

/// Frame building errors.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BuilderError {
    /// Builder output would overflow the maximum size allowed.
    Overflow,
}

/// Frame Parsing Errors
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParsingError {
    /// Invalid StreamId
    InvalidStreamId,
    /// Parse payload was called with invalid state
    ExpectedStatePayload,
    /// Parse was called with invalid state
    InvalidState,
}

/// H2 Settings Error
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum H2SettingsError {
    /// Multiple Payloads when only one allowed
    DuplicatePayload,
    /// Invalid settings payload
    InvalidPayload,
    /// Enable push must be either 0 or 1.
    InvalidEnablePush,
    /// Maximum initial window size is 2^31-1.
    /// This must be treated as Flow Control Error
    FlowControlError,
    /// Maximum allowed frame size must be between 2^14 and 2^24-1.
    /// This must be treated as Protcol Error
    ProtocolError,
}
