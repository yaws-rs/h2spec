//! H2 Flags
/*****************************
  Unused Flags (4),    0-3   7-4
  PADDED Flag (1),     4     3
  Unused Flags (2),    5,6   2,1
  END_STREAM Flag (1), 7     0
******************************/
use crate::FrameFlags;

/// Data flags
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct DataFlags(u8);

impl From<u8> for DataFlags {
    fn from(f: u8) -> Self {
        Self(f)
    }
}

impl From<FrameFlags> for DataFlags {
    fn from(f: FrameFlags) -> Self {
        Self(f.as_u8())
    }
}

impl DataFlags {
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
        let hdr_flags: DataFlags = flags.into();

        assert_eq!(hdr_flags.end_stream(), true);
        assert_eq!(hdr_flags.padded(), false);
    }
}
