//! H2 Flags
/*****************************
  Unused Flags (2),     0, 1
  PRIORITY Flag (1),    2
  Unused Flag (1),      3
  PADDED Flag (1),      4
  END_HEADERS Flag (1), 5
  Unused Flag (1),      6
  END_STREAM Flag (1),  7
******************************/
use crate::FrameFlags;

/// Header flags
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
    /// Priority flag?
    pub fn priority(&self) -> bool {
        self.0 >> 2 & 1
    }
    pub fn end_headers(&self) -> bool {
        self.0 >> 5 & 1
    }
    pub fn end_stream(&self) -> bool {
        self.0 >> 7 & 1
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn basic() {
        let flags = 5;
        let hdr_flags: HeaderFlags = flags.into();

        assert_eq!(hdr_flags.priority(), false);
        assert_eq!(hdr_flags.end_stream(), false);
        assert_eq!(hdr_flags.end_headers(), false);        
    }
}

