//! Next frame out

use super::*;
use crate::MyH2App;
use blueprint_h2spec::H2DataFlags;

use log::debug;

impl<App> MyH2App<App>
where
    App: ResponseSender
        + RequestReceiver
        + RequestHeaderReceiver
        + RequestBodyReceiver
        + Default
        + Reset,
{
    fn try_next_external_meta(&mut self) -> Option<H2FrameMeta> {
        let mut chosen_stream: Option<u32> = None;
        let mut remaining = 0;
        let mut fc_max: u32 = 0;
        for (stream_id, fc) in self.fc_streams.iter() {
            if fc.remaining > 0 {
            //if fc.remaining > 0 && fc.max > 0 {
                chosen_stream = Some(*stream_id);
                fc_max = fc.max;
                remaining = fc.remaining;
                break;
            }
        }

        let chosen_stream = match chosen_stream {
            Some(sid) => sid,
            None => return None,
        };

        debug!(
            "Chosen stream {} with headroom {} & remaining {}",
            chosen_stream, fc_max, remaining
        );

        let app = match self.fc_frames_out.get_mut(&chosen_stream) {
            Some(app) => app,
            None => unreachable!("No App in stream_id {chosen_stream}"),
        };

        let mut end_stream = false;

        fc_max = 8000;

        let send_size = if fc_max as usize >= remaining {
            end_stream = true;
            remaining as u32
        } else {
            fc_max
        };

        let mut payload_body = Vec::with_capacity(send_size as usize);
        let payload_copy = app.resp_on_body(send_size as usize);
        payload_body.extend(payload_copy);

        let mut data_out_flags = H2DataFlags::new();
        if end_stream == true {
            data_out_flags.set_end_stream(true);
        }
        let data_flags_u8 = data_out_flags.as_u8();

        let frame_out = InternalFrameOut {
            stream_id: StreamId::unchecked_new(chosen_stream),
            frame_type: FrameType::Data,
            frame_flags: FrameFlags::from_u8(data_flags_u8),
            payload: payload_body,
        };

        let mut meta = H2FrameMeta::new(FrameType::Data);
        meta.set_flags(frame_out.frame_flags);
        meta.set_stream_id(StreamId::unchecked_new(chosen_stream));
        let _ = meta.set_payload_len(send_size);

        if let Some(fc) = self.fc_streams.get_mut(&chosen_stream) {
            fc.remaining -= send_size as usize;
            fc.consumed += send_size;
        }

        self.cur_frame_out = Some(frame_out);
        Some(meta)
    }
    fn try_next_internal_meta(&mut self) -> Option<H2FrameMeta> {
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
    pub(crate) fn try_next_frame_meta(&mut self) -> Option<H2FrameMeta> {
        if let Some(internal_meta) = self.try_next_internal_meta() {
            return Some(internal_meta);
        }
        if let Some(external_meta) = self.try_next_external_meta() {
            return Some(external_meta);
        }
        None
    }
}
