//! H2App relaying H11App

mod cur;
use cur::*;

use blueprint_h2spec::H2ErrorOut;
use blueprint_h2spec::{
    H2FrameFlags as FrameFlags, H2FrameType as FrameType, H2StreamId as StreamId,
};
use blueprint_h2spec::{H2FrameMeta, H2FrameReceiver, H2FrameSender, H2Reset};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
enum H2Expect {
    #[default]
    ClientSettings,
    Nothing,
}

#[derive(Clone, Debug, PartialEq)]
struct InternalFrameOut {
    stream_id: StreamId,
    frame_type: FrameType,
    frame_flags: FrameFlags,
    payload: Vec<u8>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct MyH2App {
    expect: H2Expect,
    // We expect we receive these in order
    cur_receive: Option<CurFrameReceiving>,
    internal_frames_out: Vec<InternalFrameOut>,
    cur_frame_out: Option<InternalFrameOut>,
}

impl H2Reset for MyH2App {
    #[inline]
    fn reset(&mut self) {
        // We should probably nuke this
    }
}

impl H2FrameSender for MyH2App {
    fn send_start_meta(&mut self) -> Option<H2FrameMeta> {
        let frame_out = match self.internal_frames_out.pop() {
            Some(frame) => frame,
            None => return None,
        };

        let mut meta = H2FrameMeta::new(frame_out.frame_type);
        meta.set_flags(frame_out.frame_flags);
        meta.set_stream_id(frame_out.stream_id);
        let _ = meta.set_payload_len(frame_out.payload.len() as u32);

        self.cur_frame_out = Some(frame_out);

        Some(meta)
    }
    fn send_payload(&mut self, payload: &mut [u8]) -> usize {
        let cur = match self.cur_frame_out {
            Some(ref mut cur) => cur,
            None => return 0,
        };

        if payload.len() == 0 && cur.payload.len() == 0 {
            return 0;
        }

        let copy_len = if payload.len() < cur.payload.len() {
            payload.len()
        } else {
            cur.payload.len()
        };
        payload[..copy_len].copy_from_slice(&cur.payload[..copy_len]);

        let new_slice = &cur.payload[copy_len..];
        let mut new_payload = Vec::with_capacity(new_slice.len());
        new_payload.copy_from_slice(&new_slice);

        cur.payload = new_payload;

        copy_len
    }
}

impl MyH2App {
    #[inline]
    fn handle_window_update(&mut self) -> Result<(), H2ErrorOut> {
        let cur = self.get_cur(FrameType::WindowUpdate)?;
        println!(
            "StreamId[{:?}] WINDOW_UPDATE Flags/{:?} Buf[{}]={:?}",
            cur.stream_id,
            cur.frame_flags,
            cur.internal_buffer.len(),
            &cur.internal_buffer
        );
        Ok(())
    }
    #[inline]
    fn handle_settings(&mut self) -> Result<(), H2ErrorOut> {
        let cur = self.get_cur(FrameType::Settings)?;

        println!(
            "StreamId[{:?}] SETTINGS Flags/{:?} Buf[{}]={:?}",
            cur.stream_id,
            cur.frame_flags,
            cur.internal_buffer.len(),
            &cur.internal_buffer
        );
        Ok(())
    }
    #[inline]
    fn handle_go_away(&mut self) -> Result<(), H2ErrorOut> {
        let cur = self.get_cur(FrameType::GoAway)?;

        println!(
            "StreamId[{:?}] GO AWAY Flags/{:?} Buf[{}]={:?}",
            cur.stream_id,
            cur.frame_flags,
            cur.internal_buffer.len(),
            &cur.internal_buffer
        );
        Ok(())
    }
}

impl H2FrameReceiver for MyH2App {
    #[inline]
    fn header(
        &mut self,
        frame_type: FrameType,
        frame_flags: FrameFlags,
        stream_id: StreamId,
        expect_len: u32,
    ) -> Result<(), H2ErrorOut> {
        if self.expect == H2Expect::ClientSettings {
            if frame_type != FrameType::Settings {
                return Err(H2ErrorOut::Protocol);
            }
        }

        self.cur_receive = match self.cur_receive {
            None => Some(CurFrameReceiving {
                frame_type,
                frame_flags,
                stream_id,
                expect_len,
                internal_buffer: vec![],
            }),
            Some(_) => {
                return Err(H2ErrorOut::Internal);
            }
        };

        //println!("Cur Receive = {:?}", self.cur_receive);

        Ok(())
    }
    fn payload(&mut self, payload: &[u8], remaining: u32) -> Result<(), H2ErrorOut> {
        //println!("Payload len {} - remaining {}", payload.len(), remaining);

        let mut internal: Option<FrameType> = None;
        let mut external: Option<FrameType> = None;

        if self.cur_is_internal() {
            let cur = self.get_cur_mut()?;

            cur.internal_buffer.extend(payload);
            internal = Some(cur.frame_type);
        } else {
            let cur = self.get_cur_any()?;
            external = Some(cur.frame_type);
        }

        if let Some(frame_type) = internal
            && remaining == 0
        {
            match frame_type {
                FrameType::Settings => self.handle_settings()?,
                FrameType::WindowUpdate => self.handle_window_update()?,
                FrameType::GoAway => self.handle_go_away()?,
                _ => {}
            }
        }

        if let Some(frame_type) = external {
            match frame_type {
                FrameType::Settings => unreachable!(),
                FrameType::WindowUpdate => unreachable!(),
                _ => todo!("Need to handle Self = {:?} for payload", self),
            }
        }

        if remaining == 0 {
            if self.expect == H2Expect::ClientSettings {
                self.expect = H2Expect::Nothing;
            }
            self.cur_receive = None;
        }

        Ok(())
    }
}
