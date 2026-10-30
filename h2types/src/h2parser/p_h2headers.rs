//! FrameType::Headers Payload streaming parser

use super::Remaining;
use crate::Discard;

/// What the headers parser currently expects
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum HeadersExpect {
    /// Metadata
    #[default]
    Meta,
    /// Field Block
    FieldBlock,
    /// Padding
    Padding(Remaining),
}

/// H2 Headers Parsing Status
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum HeadersParsingStatus {
    /// Parsing is initialized but not run
    #[default]
    Initialized,
    /// Parsing is incomplete at Expect with Discard bytes
    Incomplete(HeadersExpect, Discard),
    /// Parsing is complete at Expect with Discard bytes
    Complete(HeadersExpect, Discard),
}

/// Headers payload parser
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct HeadersParser;

impl HeadersParser {
    /// New stateful instance
    pub fn new() -> Self {
        Self
    }
    /// Parse
    pub fn parse(&mut self) -> HeadersParsingStatus {
        todo!()
    }
}
