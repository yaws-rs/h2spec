//! H2App relaying H11App

mod cur;
mod frame_out;
mod headers;

use cur::*;

use blueprint_h2spec::H2ErrorOut;
use blueprint_h2spec::{
    H2FrameFlags as FrameFlags, H2FrameType as FrameType, H2StreamId as StreamId,
};
use blueprint_h2spec::{H2FrameMeta, H2FrameReceiver, H2FrameSender};

use blueprint_h11spec::{
    RequestBodyReceiver, RequestHeaderReceiver, RequestReceiver, Reset, ResponseSender,
};

use log::debug;

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

use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq)]
struct FlowControl {
    consumed: u32,
    remaining: usize,
    max: u32,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct MyH2App<App> {
    expect: H2Expect,
    // We expect we receive these in order
    cur_receive: Option<CurFrameReceiving>,
    internal_frames_out: Vec<InternalFrameOut>,
    cur_frame_out: Option<InternalFrameOut>,
    app: App,
    fc_frames_out: HashMap<u32, App>,
    fc_streams: HashMap<u32, FlowControl>,
}

impl<App> H2FrameSender for MyH2App<App>
where
    App: ResponseSender
        + RequestReceiver
        + RequestHeaderReceiver
        + RequestBodyReceiver
        + Default
        + Reset,
{
    fn send_start_meta(&mut self) -> Option<H2FrameMeta> {
        self.try_next_frame_meta()
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
        new_payload.extend(new_slice);

        cur.payload = new_payload;

        copy_len
    }
}

impl<App> MyH2App<App>
where
    App: ResponseSender
        + RequestReceiver
        + RequestHeaderReceiver
        + RequestBodyReceiver
        + Default
        + Reset,
{
    #[inline]
    fn handle_window_update(&mut self) -> Result<(), H2ErrorOut> {
        let cur = self.get_cur(FrameType::WindowUpdate)?;
        debug!(
            "StreamId[{:?}] WINDOW_UPDATE Flags/{:?} Buf[{}]={:?}",
            cur.stream_id,
            cur.frame_flags,
            cur.internal_buffer.len(),
            &cur.internal_buffer
        );

        let b_size = [
            cur.internal_buffer[0],
            cur.internal_buffer[1],
            cur.internal_buffer[2],
            cur.internal_buffer[3],
        ];

        let fc_len = u32::from_be_bytes(b_size);

        let mut fc_stream = self.fc_streams.get_mut(&cur.stream_id.as_u32());
        let flow_control = match fc_stream {
            Some(ref mut fc) => fc,
            None => return Ok(()),
        };

        flow_control.max = fc_len;

        debug!("Updating to window to {fc_len}");

        Ok(())
    }
    #[inline]
    fn handle_settings(&mut self) -> Result<(), H2ErrorOut> {
        let cur = self.get_cur_mut(FrameType::Settings)?;

        debug!(
            "StreamId[{:?}] SETTINGS Flags/{:?} Buf[{}]={:?}",
            cur.stream_id,
            cur.frame_flags,
            cur.internal_buffer.len(),
            &cur.internal_buffer
        );

        if cur.frame_flags.as_u8() == 0 {
            let ack_out = InternalFrameOut {
                stream_id: StreamId::unchecked_new(0),
                frame_type: FrameType::Settings,
                frame_flags: FrameFlags::from_u8(1),
                payload: vec![],
            };

            self.internal_frames_out.push(ack_out);
        }

        Ok(())
    }
    #[inline]
    fn handle_go_away(&mut self) -> Result<(), H2ErrorOut> {
        let cur = self.get_cur(FrameType::GoAway)?;

        debug!(
            "StreamId[{:?}] GO AWAY Flags/{:?} Buf[{}]={:?}",
            cur.stream_id,
            cur.frame_flags,
            cur.internal_buffer.len(),
            &cur.internal_buffer
        );

        Err(H2ErrorOut::Protocol)
    }
}

impl<App> H2FrameReceiver for MyH2App<App>
where
    App: ResponseSender
        + RequestReceiver
        + RequestHeaderReceiver
        + RequestBodyReceiver
        + Default
        + Reset,
{
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

        if expect_len == 0 {
            self.cur_receive = None;
        }

        Ok(())
    }
    fn payload(&mut self, payload: &[u8], remaining: u32) -> Result<(), H2ErrorOut> {
        let mut internal: Option<FrameType> = None;
        let mut external: Option<FrameType> = None;

        if self.cur_is_internal() {
            let cur = self.get_cur_mut_any()?;

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
                _ => todo!("Handle Internal frame {:?}", frame_type),
            }
        }

        if let Some(frame_type) = external {
            match frame_type {
                FrameType::Settings => unreachable!(),
                FrameType::WindowUpdate => unreachable!(),
                FrameType::Headers => self.handle_headers(payload, remaining)?,
                FrameType::RstStream => {
                    // Shutdown
                    return Ok(());
                }
                _ => todo!("Need to handle FrameType = {:?} for payload", frame_type),
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
