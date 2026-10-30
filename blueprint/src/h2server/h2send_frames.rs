//! Send H2 Frames

type Remaining = u32;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub(crate) enum SendStage {
    #[default]
    Header,
    Payload(Remaining),
}

use super::*;

impl<App> H2Serving<App>
where
    App: H2FrameSender + H2FrameReceiver,
{
    pub(crate) fn send_h2frames(&mut self, mut remaining: &mut [u8]) -> StageSenderStatus {
        let mut filled = 0;

        loop {
            if (self.send_stage == SendStage::Header && remaining.len() < 9) || remaining.len() == 0
            {
                break;
            }
            if self.send_stage == SendStage::Header {
                let header = match self.app.send_start_meta() {
                    Some(hdr) => hdr,
                    None => return StageSenderStatus::with_ok(filled),
                };

                let bytes_out = match header.as_bytes(remaining) {
                    Some(bytes_out) => bytes_out,
                    None => return StageSenderStatus::with_ok(filled),
                };
                filled += bytes_out.len();
                remaining = &mut remaining[9..];
                self.send_stage = SendStage::Payload(header.payload_len());
            }

            if let SendStage::Payload(send_remaining) = self.send_stage {
                if send_remaining == 0 {
                    self.send_stage = SendStage::Header;
                    continue;
                }

                let n_expect = if send_remaining < remaining.len() as u32 {
                    send_remaining
                } else {
                    remaining.len() as u32
                };

                let n_filled = self.app.send_payload(&mut remaining[..n_expect as usize]);
                if n_filled > remaining.len() {
                    return StageSenderStatus::with_error(H2ErrorOut::Internal, n_filled);
                }
                filled += n_filled;
                remaining = &mut remaining[n_filled..];

                let payload_remaining = send_remaining - n_filled as u32;
                self.send_stage = match payload_remaining {
                    0 => SendStage::Header,
                    _ => SendStage::Payload(payload_remaining),
                };
            }
        }

        StageSenderStatus::with_ok(filled)
    }
}
