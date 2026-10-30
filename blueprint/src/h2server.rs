//! h11spec Server blueprints

mod h11upgrade;
mod h2framing;
mod h2send_frames;
mod h2settings;

use h11upgrade::H11Upgrade;
use h2send_frames::SendStage;
use h2settings::SettingsFrameOut;

use crate::H2Error;
use crate::{borrow_remaining_left_b, try_fill_left_with, FillAttempt};

use blueprint::BluePrint;
use blueprint::Orbit;
use blueprint::{InBuffer, Left, Right};

use h11types::{
    H11RequestMeta, HeaderStatus as H11HeaderStatus, RequestBodySize as H11RequestBodySize,
    StartLineStatus as H11StartLineStatus,
};

#[doc(inline)]
pub use h2types::{
    DataFlags as H2DataFlags, FrameFlags as H2FrameFlags, FrameType as H2FrameType,
    HeaderFlags as H2HeaderFlags, StreamId as H2StreamId,
};
#[doc(inline)]
pub use h2types::{H2ErrorOut, H2FrameMeta, H2FrameReceiver, H2FrameSender};

use h2types::H2Settings;

/// empty for now
pub struct Position;

// Discard N bytes from Ingress buffer
type Discard = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct StageHandlerStatus {
    pub(crate) discard: Discard,
    pub(crate) h2error: Option<H2ErrorOut>,
}

impl StageHandlerStatus {
    pub(crate) fn with_error_out(h2err: H2ErrorOut, discard: Discard) -> Self {
        Self {
            discard,
            h2error: Some(h2err),
        }
    }
    pub(crate) fn with_ok(discard: Discard) -> Self {
        Self {
            discard,
            h2error: None,
        }
    }
}

// Filled N bytes into Egress buffer
type Filled = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct StageSenderStatus {
    pub(crate) filled: Filled,
    pub(crate) error: Option<H2ErrorOut>,
}

impl StageSenderStatus {
    pub(crate) fn get_filled(&self) -> Filled {
        self.filled
    }
    pub(crate) fn get_err(&self) -> Option<H2ErrorOut> {
        self.error
    }
    pub(crate) fn with_ok(filled: Filled) -> Self {
        Self {
            filled,
            error: None,
        }
    }
    pub(crate) fn with_error(error: H2ErrorOut, filled: Filled) -> Self {
        Self {
            filled,
            error: Some(error),
        }
    }
}

type Upgraded = bool;

/// State machine stages
#[derive(Debug, Default, PartialEq)]
enum H2ServingStage {
    /// Expect Request Start line
    #[default]
    Start,
    /// H11 Upgrade headers expected
    H11UpgradeHeaders(H11Upgrade, H11RequestMeta),
    /// H11 Upgrade body to be consumed
    H11UpgradeBody(H11Upgrade, H11RequestMeta),
    /// H11 Upgrade response sending
    H11UpgradeResponse(H11Upgrade),
    /// H2 Server Preface (Settings)
    ServerSettings(Upgraded),
    /// H2 Client Preface (Magic)
    ClientPreface,
    /// H2 Expected Client Preface but got wrong
    ExpectedClientPreface,
    /// H2 Framing
    Framing,
    /// Did not receive Preface or H11Upgrade at start
    BadStart,
    /// H11 Bad request encountered
    H11BadRequest,
    /// H11 Shutdown
    H11Shutdown,
    /// H2 Unexpected Server Error
    UnexpectedServerError,
}

/// H2Server Serving context
#[derive(Debug)]
pub struct H2Serving<App> {
    stage: H2ServingStage,
    send_stage: SendStage,
    server_h2settings: H2Settings,
    client_h2settings: H2Settings,
    app: App,
}

const CRLF: &[u8] = "\r\n".as_bytes();

// RFC 9113 s. 3.4
const PREFACE_MAGIC: &[u8] = &[
    0x50, 0x52, 0x49, 0x20, 0x2a, 0x20, 0x48, 0x54, 0x54, 0x50, 0x2f, 0x32, 0x2e, 0x30, 0x0d, 0x0a,
    0x0d, 0x0a, 0x53, 0x4d, 0x0d, 0x0a, 0x0d, 0x0a,
];
//const PREFACE_MAGIC: &[u8] = "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".as_bytes();

impl<App> Orbit for H2Serving<App>
where
    App: H2FrameSender + H2FrameReceiver,
{
    type Position = Position;
    type Error = H2Error;
    fn advance_with<B, L: Left, R: Right>(
        &mut self,
        _u: &mut B,
        l: &mut L,
        _r: &mut R,
    ) -> Result<Self::Position, Self::Error> {
        let (left_in_len, left_out_len) = l.left_lens();
        let (left_inputs, left_out_b) = l.left_bufs_mut();

        let mut left_in_b = match left_inputs {
            InBuffer::Single(buf) => buf,
            InBuffer::Double(_buf1, _buf2) => {
                todo!()
            }
        };

        if left_in_len <= PREFACE_MAGIC.len() && self.stage == H2ServingStage::Start {
            return Ok(Position);
        }

        let mut out_len = left_out_len;
        let mut discard = 0;
        let mut shutdown = false;

        let mut ignore_framing = false;

        loop {
            match self.stage {
                H2ServingStage::Start => {
                    if left_in_b[..PREFACE_MAGIC.len()] == *PREFACE_MAGIC {
                        self.stage = H2ServingStage::ServerSettings(false);
                        let _ = left_in_b.split_off_mut(..PREFACE_MAGIC.len());
                        discard += PREFACE_MAGIC.len();
                        continue;
                    }

                    // Technically RFC 9113 deprecated this since nobody used it.
                    let mut try_upgrade = H11Upgrade::default();
                    let mut h11_meta = H11RequestMeta::default();

                    let advanced = match h11_meta.advance_status_with(&mut try_upgrade, left_in_b) {
                        Ok(H11StartLineStatus::Complete(advanced)) => advanced,
                        Ok(H11StartLineStatus::Incomplete(_, _)) => {
                            break;
                        }
                        Err(_e) => {
                            self.stage = H2ServingStage::BadStart;
                            shutdown = true;
                            break;
                        }
                    };

                    let _ = left_in_b.split_off_mut(..advanced);
                    discard += advanced;

                    if h11_meta.status_complete() {
                        self.stage = H2ServingStage::H11UpgradeHeaders(try_upgrade, h11_meta);
                        continue;
                    }
                }
                H2ServingStage::H11UpgradeHeaders(mut upgrade, mut h11_meta) => {
                    match h11_meta.advance_headers_with(&mut upgrade, left_in_b) {
                        Ok(H11HeaderStatus::Incomplete(b, _))
                        | Ok(H11HeaderStatus::Complete(b)) => {
                            discard += b;
                            let _ = left_in_b.split_off_mut(..b);
                        }
                        Err(_e) => {
                            self.stage = H2ServingStage::H11BadRequest;
                            continue;
                        }
                    };

                    if h11_meta.headers_complete() {
                        self.stage = H2ServingStage::H11UpgradeBody(upgrade, h11_meta);
                        continue;
                    }
                }
                H2ServingStage::H11UpgradeBody(upgrade, h11_meta) => {
                    if let H11RequestBodySize::Empty = h11_meta.body_size() {
                        self.stage = H2ServingStage::H11UpgradeResponse(upgrade);
                        continue;
                    }
                    if let H11RequestBodySize::Fixed(expected_len) = h11_meta.body_size() {
                        let remaining = expected_len.get() - upgrade.body_consumed();
                        let relay_size = if left_in_b.len() >= remaining {
                            self.stage = H2ServingStage::H11UpgradeResponse(upgrade);
                            remaining
                        } else {
                            left_in_b.len()
                        };
                        match left_in_b.split_off_mut(..relay_size) {
                            Some(relay_b) => {
                                discard += relay_b.len();
                            }
                            None => {}
                        }
                    } else {
                        self.stage = H2ServingStage::H11BadRequest;
                        continue;
                    }
                }
                H2ServingStage::H11UpgradeResponse(upgrade) => {
                    if !upgrade.correct_upgrade() {
                        self.stage = H2ServingStage::H11BadRequest;
                        continue;
                    }

                    self.client_h2settings = upgrade.settings();

                    let remaining = borrow_remaining_left_b(left_out_b, out_len, 22);
                    let switching_protocols =
                        "HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: h2c\r\n".as_bytes();

                    let filled = match try_fill_left_with(remaining, &[switching_protocols, CRLF]) {
                        FillAttempt::Complete(filled) => filled,
                        FillAttempt::Overflow => break,
                    };
                    out_len += filled;

                    self.stage = H2ServingStage::ServerSettings(true);
                    continue;
                }
                H2ServingStage::ServerSettings(upgraded) => {
                    let mut framer = SettingsFrameOut::default();

                    match self.server_h2settings.frame_out(&mut framer) {
                        Ok(()) => {}
                        _ => {
                            self.stage = H2ServingStage::UnexpectedServerError;
                            continue;
                        }
                    }

                    let settings_frame = match framer.frame_as_bytes() {
                        Some(frame) => frame,
                        None => {
                            self.stage = H2ServingStage::UnexpectedServerError;
                            continue;
                        }
                    };

                    let remaining = borrow_remaining_left_b(left_out_b, out_len, 22);
                    let filled = match try_fill_left_with(remaining, &[settings_frame]) {
                        FillAttempt::Complete(filled) => filled,
                        FillAttempt::Overflow => break,
                    };
                    out_len += filled;

                    self.stage = match upgraded {
                        true => H2ServingStage::ClientPreface,
                        false => H2ServingStage::Framing,
                    };
                    ignore_framing = true;
                    break;
                }
                H2ServingStage::ClientPreface => {
                    if left_in_b.len() <= PREFACE_MAGIC.len() {
                        break;
                    }

                    if left_in_b[..PREFACE_MAGIC.len()] == *PREFACE_MAGIC {
                        let _ = left_in_b.split_off_mut(..PREFACE_MAGIC.len());
                        discard += PREFACE_MAGIC.len();
                        self.stage = H2ServingStage::Framing;
                        continue;
                    } else {
                        self.stage = H2ServingStage::ExpectedClientPreface;
                        shutdown = true;
                        break;
                    }
                }
                H2ServingStage::Framing => {
                    break;
                }
                H2ServingStage::BadStart | H2ServingStage::ExpectedClientPreface => {
                    shutdown = true;
                    break;
                }
                H2ServingStage::UnexpectedServerError => {
                    shutdown = true;
                    break;
                }
                H2ServingStage::H11BadRequest => {
                    let remaining = borrow_remaining_left_b(left_out_b, 0, 22);

                    let bad_request =
                        "HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n".as_bytes();

                    let filled = match try_fill_left_with(remaining, &[bad_request, CRLF]) {
                        FillAttempt::Complete(filled) => filled,
                        FillAttempt::Overflow => break,
                    };
                    out_len += filled;
                    self.stage = H2ServingStage::H11Shutdown;
                    break;
                }
                H2ServingStage::H11Shutdown => {
                    break;
                }
            }
        }

        if ignore_framing == false && self.stage == H2ServingStage::Framing {
            let h2_frames_res = self.handle_h2frames(left_in_b);
            discard += h2_frames_res.discard;

            let remaining = borrow_remaining_left_b(left_out_b, out_len, 22);
            let send_status = self.send_h2frames(remaining);

            if send_status.get_filled() > 0 {
                out_len += send_status.get_filled();
            }

            if let Some(_err) = send_status.get_err() {
                l.shutdown();
            }
        }

        let new_in_len = left_in_len - discard;

        l.left_set_lens(new_in_len, out_len);

        if shutdown {
            l.shutdown();
        }

        Ok(Position)
    }
}

/// H2Server which implements the [`Orbit`] and [`BluePrint`] traits.
pub struct H2SpecServer;
/// Placeholder for H2ServerConfig
pub struct H2ServerConfig;

impl<App> Default for H2Serving<App>
where
    App: H2FrameSender + H2FrameReceiver + Default,
{
    fn default() -> Self {
        let mut server_h2settings = H2Settings::default();
        let _ = server_h2settings.set_enable_push(0u32);

        Self {
            stage: H2ServingStage::default(),
            send_stage: SendStage::default(),
            server_h2settings,
            client_h2settings: H2Settings::default(),
            app: App::default(),
        }
    }
}

impl<App> H2Serving<App>
where
    App: H2FrameSender + H2FrameReceiver + Default,
{
    /// Initialize H2Serving with the specified pre-initialized App
    pub fn with_app(app: App) -> Self {
        let mut server_h2settings = H2Settings::default();
        let _ = server_h2settings.set_enable_push(0u32);

        Self {
            stage: H2ServingStage::default(),
            send_stage: SendStage::default(),
            server_h2settings,
            client_h2settings: H2Settings::default(),
            app,
        }
    }
}

impl<App> BluePrint<H2Serving<App>> for H2SpecServer
where
    App: H2FrameSender + H2FrameReceiver + Default,
{
    type Config = H2ServerConfig;
    type Error = H2Error;

    fn with_defaults() -> Result<H2Serving<App>, Self::Error> {
        Ok(H2Serving::default())
    }
    fn with_configuration(_: Self::Config) -> Result<H2Serving<App>, Self::Error> {
        todo!()
    }
}
