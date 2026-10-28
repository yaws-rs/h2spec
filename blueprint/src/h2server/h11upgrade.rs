//! h11spec Upgrade Handling

use h11types::H11Connection;
use h11types::H11RequestMeta;

use h11types::{H11Header, H11MaybeValue, H11Method, H11TransferEncoding, H11Version, Uri};
use h11types::{
    NoRequestTargetReceiver, RequestBodyReceiver, RequestHeaderReceiver, RequestReceiver,
    RequestTargetReceiver,
};
use h11types::{
    Resp2xx, RespComplete, RespIndicative, ResponseBodySize, ResponseSender, StatusDesc,
};

use crate::H2Serving;
use h2types::H2Settings;
use h2types::H2SettingsError;

// For HTTP2-Settings
use base64::{engine::general_purpose, Engine};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(super) struct H11Upgrade {
    settings: H2Settings,
    error_settings: Option<H2SettingsError>,
    correct_settings: bool,
    correct_connection: bool,
    correct_upgrade: bool,
    body_consumed: usize,
}

impl H11Upgrade {
    // Is the upgrade correct?
    pub(super) fn correct_upgrade(&self) -> bool {
        self.correct_settings
            && self.correct_connection
            && self.correct_upgrade
            && self.error_settings == None
    }
    pub(super) fn settings(&self) -> H2Settings {
        self.settings.clone()
    }
    pub(super) fn body_consumed(&self) -> usize {
        self.body_consumed
    }
}

impl RequestReceiver for H11Upgrade {
    fn req_method(&mut self, _mthd: H11Method) {}
    fn impl_target(&mut self) -> impl RequestTargetReceiver {
        NoRequestTargetReceiver
    }
    fn req_version(&mut self, _version: H11Version) {}
    fn req_start_headers(&mut self) -> RespIndicative {
        RespIndicative::GoAhead
    }
    fn req_start_payload(&mut self) -> RespIndicative {
        RespIndicative::GoAhead
    }
    fn req_complete(&mut self) -> RespIndicative {
        RespIndicative::GoAhead
    }
}

impl RequestHeaderReceiver for H11Upgrade {
    /// Single Header                                                                                             
    fn req_header<'h>(&mut self, hdr: H11Header<'h>) -> RespIndicative {
        match hdr {
            H11Header::Http2Settings(H11MaybeValue::Bytes(b)) => {
                if self.correct_settings {
                    self.error_settings = Some(H2SettingsError::DuplicatePayload);
                    self.correct_settings = false;
                    return RespIndicative::GoAhead;
                }
                let mut b64_out: [u8; 60] = [0; 60];
                let len = match general_purpose::STANDARD.decode_slice(&b, &mut b64_out) {
                    Ok(len) => len,
                    Err(_) => {
                        self.error_settings = Some(H2SettingsError::InvalidPayload);
                        return RespIndicative::GoAhead;
                    }
                };

                self.settings = match H2Settings::from_payload(&b64_out[..len]) {
                    Err(_) => {
                        self.error_settings = Some(H2SettingsError::InvalidPayload);
                        return RespIndicative::GoAhead;
                    }
                    Ok(ret) => ret,
                };
                self.correct_settings = true;
            }
            H11Header::Connection(c) if c == H11Connection::Http2Upgrade => {
                self.correct_connection = true;
            }
            H11Header::Upgrade(H11MaybeValue::Bytes(u)) if u == &[104, 50, 99] => {
                self.correct_upgrade = true;
            }
            _ => {}
        }
        RespIndicative::GoAhead
    }
    /// Headers processing has finished                                                                           
    fn req_headers_finish(&mut self) -> RespIndicative {
        RespIndicative::GoAhead
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use rstest::*;

    struct TestApp;
    impl Reset for TestApp {
        fn reset(&mut self) {}
    }

    #[rstest]
    #[case("GET / HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/8.11.1\r\nAccept: */*\r\nConnection: Upgrade, HTTP2-Settings\r\nUpgrade: h2c\r\nHTTP2-Settings: AAMAAABkAAQAAQAAAAIAAAAA\r\n\r\n")]
    fn upgrade(#[case] input: &'static str) {
        let mut h2 = H2Serving::with_app(TestApp);
    }
}
