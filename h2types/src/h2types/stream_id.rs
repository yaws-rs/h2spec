//! Stream Identifier

/// H2 31 bit Stream Identifier. Max 2,147,483,647
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct StreamId(u32);

impl StreamId {
    /// Construct new StreamId
    pub fn new(input: u32) -> Option<Self> {
        if input <= 2_147_483_647 {
            return Some(Self(input));
        }
        None
    }

    /// Construct new unchecked StreamId
    pub fn unchecked_new(input: u32) -> Self {
        Self(input)
    }
    /// As u32
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
