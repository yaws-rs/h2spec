//! Handle headers

use super::*;
use blueprint_h2spec::H2HeaderFlags;

use blueprint_h11spec::{H11Header, H11Method};
use blueprint_h11spec::{
    RequestBodyReceiver, RequestHeaderReceiver, RequestReceiver, RequestTargetReceiver, Reset,
    ResponseSender,
};

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
    #[inline]
    pub(crate) fn handle_headers(
        &mut self,
        payload: &[u8],
        remaining: u32,
    ) -> Result<(), H2ErrorOut> {
        let cur = self.get_cur(FrameType::Headers)?;

        let stream_id = cur.stream_id;
        let header_flags: H2HeaderFlags = cur.frame_flags.into();

        debug!(
            "StreamId[{:?}] HEADERS Flags/{:?} Priority/{:?} Padded/{:?} EndStream/{:?} EndHeaders/{:?} / Remaining={:?}  Buf[{}]={:?}",
            cur.stream_id,
            cur.frame_flags,
            header_flags.priority(),
            header_flags.padded(),
            header_flags.end_stream(),
            header_flags.end_headers(),
            remaining,
            payload.len(),
            payload,
        );

        let mut decoder = loona_hpack::Decoder::new();

        let hdrs = match decoder.decode(payload) {
            Ok(hdrs) => hdrs,
            Err(_) => return Err(H2ErrorOut::Internal),
        };

        enum Scheme {
            Http,
        }

        // Reconstruct into HTTP/1.1 Request
        let mut method: Option<H11Method> = None;
        let mut _scheme: Option<Scheme> = None;
        let mut authority: Option<Vec<u8>> = None;
        let mut path: Option<Vec<u8>> = None;
        let mut header_payload: Vec<u8> = vec![];

        for hdrkv in &hdrs {
            let v = match core::str::from_utf8(&hdrkv.1) {
                Ok(v) => v,
                Err(_) => return Err(H2ErrorOut::Internal),
            };

            match hdrkv.0.as_slice() {
                b":method" => {
                    let m = match H11Method::try_from(hdrkv.1.as_slice()) {
                        Ok(m) => m,
                        Err(_e) => {
                            return Err(H2ErrorOut::Internal);
                        }
                    };
                    method = Some(m);
                }
                b":scheme" if hdrkv.1 == b"http" => {
                    _scheme = Some(Scheme::Http);
                }
                b":authority" => {
                    authority = Some(hdrkv.1.clone());
                    header_payload.extend(format!("Host: {}\r\n", v).into_bytes());
                }
                b":path" => {
                    path = Some(hdrkv.1.clone());
                }
                _ => {
                    let k = match core::str::from_utf8(&hdrkv.0) {
                        Ok(k) => k,
                        Err(_) => return Err(H2ErrorOut::Internal),
                    };
                    header_payload.extend(format!("{}: {}\r\n", k, v).into_bytes());
                }
            }
        }

        let method = match method {
            Some(m) => m,
            None => return Err(H2ErrorOut::Internal),
        };
        let _authority = match authority {
            Some(a) => a,
            None => return Err(H2ErrorOut::Internal),
        };
        let path = match path {
            Some(p) => p,
            None => return Err(H2ErrorOut::Internal),
        };

        let mut app = App::default();
        app.req_method(method);
        let mut tgt = app.impl_target();
        tgt.req_target_init(&path);
        drop(tgt);
        app.req_version(h11types::H11Version::Http11);
        app.req_start_headers();

        struct HeaderRelay<'r, App> {
            app: &'r mut App,
            err: Option<h11types::RespIndicative>,
        }

        impl<'r, App> h11types::HeaderReceiver for HeaderRelay<'r, App>
        where
            App: RequestHeaderReceiver,
        {
            fn req_header<'h>(&mut self, hdr: H11Header<'h>) {
                let r = self.app.req_header(hdr);
                if r != h11types::RespIndicative::GoAhead {
                    self.err = Some(r);
                }
            }
            fn req_headers_finish(&mut self) {
                let r = self.app.req_headers_finish();
                if r != h11types::RespIndicative::GoAhead {
                    self.err = Some(r);
                }
            }
        }

        let mut relay = HeaderRelay {
            app: &mut app,
            err: None,
        };
        let mut p_h11header = h11types::HeaderParser::default();
        let _p_h11status = match p_h11header.parse(&mut relay, &header_payload) {
            Err(_) => return Err(H2ErrorOut::Internal),
            Ok(status) => status,
        };

        let _status = app.resp_on_status();
        let mut headers_out: Vec<(Vec<u8>, Vec<u8>)> = vec![(b":status".to_vec(), b"200".to_vec())];

        let header_count = app.resp_on_header_count();
        for hdr_x in 0..header_count {
            let hdr = app.resp_on_header(hdr_x);

            let (key_b, key_v) = match hdr {
                H11Header::Accept(v) => (b"accept".to_vec().to_vec(), v.as_bytes_out().to_vec()),
                H11Header::ContentLength(l) => {
                    (b"content-length".to_vec(), l.as_bytes_out().to_vec())
                }
                H11Header::SetCookie(s) => (b"set-cookie".to_vec(), s.as_bytes_out().to_vec()),
                H11Header::Unknown(uk, uv) => {
                    (uk.as_bytes_out().to_vec(), uv.as_bytes_out().to_vec())
                }
                _ => continue,
            };
            headers_out.push((key_b, key_v));
        }

        let mut header_encoder = loona_hpack::Encoder::new();
        let head_bytes_out =
            header_encoder.encode(headers_out.iter().map(|h| (&h.0[..], &h.1[..])));

        let mut header_out_flags = H2HeaderFlags::new();
        header_out_flags.set_end_headers(true);
        let hdr_u8 = header_out_flags.as_u8();

        let header_frame_out = InternalFrameOut {
            stream_id: StreamId::unchecked_new(1),
            frame_type: FrameType::Headers,
            frame_flags: FrameFlags::from_u8(hdr_u8),
            payload: head_bytes_out,
        };

        let payload_len = match app.resp_body_len() {
            blueprint_h11spec::ResponseBodySize::Fixed(u) => u,
            _ => return Err(H2ErrorOut::Internal),
        };

        let fc = FlowControl {
            consumed: 0,
            remaining: payload_len,
            max: 0,
        };

        self.fc_frames_out.insert(stream_id.as_u32(), app);
        self.fc_streams.insert(stream_id.as_u32(), fc);

        self.internal_frames_out.push(header_frame_out);

        return Ok(());
    }
}
