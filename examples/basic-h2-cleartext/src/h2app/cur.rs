//! Current Frame

use super::*;

impl MyH2App {
    #[inline]
    pub(crate) fn get_cur_any(&self) -> Result<&CurFrameReceiving, H2ErrorOut> {
        match self.cur_receive {
            Some(ref cur) => Ok(cur),
            _ => return Err(H2ErrorOut::Internal),
        }
    }
    #[inline]
    pub(crate) fn get_cur(&self, exp_type: FrameType) -> Result<&CurFrameReceiving, H2ErrorOut> {
        let cur = self.get_cur_any()?;
        if exp_type != cur.frame_type {
            return Err(H2ErrorOut::Internal);
        }
        Ok(cur)
    }
    #[inline]
    pub(crate) fn cur_is_internal(&self) -> bool {
        match self.cur_receive {
            Some(ref c) => c.is_internal(),
            None => false,
        }
    }
    #[inline]
    pub(crate) fn get_cur_mut(&mut self) -> Result<&mut CurFrameReceiving, H2ErrorOut> {
        match self.cur_receive {
            Some(ref mut c) => Ok(c),
            None => Err(H2ErrorOut::Internal),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CurFrameReceiving {
    pub(crate) frame_type: FrameType,
    pub(crate) frame_flags: FrameFlags,
    pub(crate) stream_id: StreamId,
    pub(crate) expect_len: u32,
    // We will not buffer large payloads but relay all of it to H11App.
    // This is only used for internal handling.
    pub(crate) internal_buffer: Vec<u8>,
}

impl CurFrameReceiving {
    pub(crate) fn is_internal(&self) -> bool {
        self.frame_type == FrameType::Settings
            || self.frame_type == FrameType::WindowUpdate
            || self.frame_type == FrameType::GoAway
    }
}
