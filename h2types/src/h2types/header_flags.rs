//! H2 Flags
/*****************************
  Unused Flags (2),     0,1  6,7
  PRIORITY Flag (1),    2    5
  Unused Flag (1),      3    4
  PADDED Flag (1),      4    3
  END_HEADERS Flag (1), 5    2
  Unused Flag (1),      6    1
  END_STREAM Flag (1),  7    0
******************************/
use crate::FrameFlags;

/// Header flags
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct HeaderFlags(u8);

impl From<u8> for HeaderFlags {
    fn from(f: u8) -> Self {
        Self(f)
    }
}

impl From<FrameFlags> for HeaderFlags {
    fn from(f: FrameFlags) -> Self {
        Self(f.as_u8())
    }
}

impl HeaderFlags {
    /// New with no flags set
    pub fn new() -> Self {
        Self(0)
    }
    /// Padded flag
    pub fn padded(&self) -> bool {
        (1 << 3 & self.0) > 0
    }
    /// Set padded flag
    pub fn set_padded(&mut self, v: bool) {
        self.set_bit(3, v);
    }
    /// Priority flag
    pub fn priority(&self) -> bool {
        (1 << 5 & self.0) > 0
    }
    /// Set priority flag
    pub fn set_priority(&mut self, v: bool) {
        self.set_bit(5, v);
    }
    /// End headers flag
    pub fn end_headers(&self) -> bool {
        (1 << 2 & self.0) > 0
    }
    /// Set end headers flag
    pub fn set_end_headers(&mut self, v: bool) {
        self.set_bit(2, v); // should be 2
    }
    /// End stream flag
    pub fn end_stream(&self) -> bool {
        (1 << 0 & self.0) > 0
    }
    /// Set end stream flag
    pub fn set_end_stream(&mut self, v: bool) {
        self.set_bit(0, v);
    }
    /// As raw byte
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    /// Set raw bit
    pub fn set_bit(&mut self, idx: u8, b: bool) -> u8 {
        let mask = !(1 << idx);
        let flag = (b as u8) << idx;
        self.0 = self.0 & mask | flag;
        self.0
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn basic() {
        let flags = 5;
        let hdr_flags: HeaderFlags = flags.into();

        assert_eq!(hdr_flags.priority(), false);
        assert_eq!(hdr_flags.end_stream(), true);
        assert_eq!(hdr_flags.end_headers(), true);
        assert_eq!(hdr_flags.padded(), false);
    }
}
