//! Send H2 Frames

/*
let remaining = borrow_remaining_left_b(left_out_b, 0, 22);
let bad_request =
    "HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n".as_bytes();

let filled = match try_fill_left_with(remaining, &[bad_request, CRLF]) {
    FillAttempt::Complete(filled) => filled,
    FillAttempt::Overflow => break,
};
out_len += filled;
*/

type Remaining = u32;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub(crate) enum SendStage {
    #[default]
    Header,
    Payload(Remaining),
}

use super::*;
use crate::{borrow_remaining_left_b, try_fill_left_with, FillAttempt};

impl<App> H2Serving<App>
where
    App: H2FrameSender + H2FrameReceiver + H2Reset,
{
    pub(crate) fn send_h2frames(&mut self, mut remaining: &mut [u8]) -> StageSenderStatus {
        let mut filled = 0;

        loop {
            if remaining.len() < 10 {
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
            }
        }

        StageSenderStatus::with_ok(filled)
    }
}
