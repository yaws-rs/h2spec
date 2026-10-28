//! H2 Settings

use crate::BuilderError;
use crate::FrameFiller;
use crate::FrameReceiver;
use crate::H2SettingsError;
use crate::SettingsFrameBuilder;

/// H2 Settings RFC 9113 s. 6.5.2. Defined Settings
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct H2Settings {
    header_table_size: u32,      // 0x01
    enable_push: bool,           // 0x02
    max_concurrent_streams: u32, // 0x03
    initial_window_size: u32,    // 0x04
    max_frame_size: u32,         // 0x05
    max_header_list_size: u32,   // 0x06
}

impl Default for H2Settings {
    /// Default H2 Settings
    fn default() -> Self {
        Self {
            header_table_size: 4_096,
            enable_push: true,
            max_concurrent_streams: 100,
            initial_window_size: 65_535,
            max_frame_size: 16_384,
            max_header_list_size: 0,
        }
    }
}

impl H2Settings {
    /// Set SETTINGS_HEADER_TABLE_SIZE (0x01)
    pub fn set_header_table_size(&mut self, v: u32) -> Result<(), H2SettingsError> {
        self.header_table_size = v;
        Ok(())
    }
    /// Set SETTINGS_ENABLE_PUSH (0x02)
    pub fn set_enable_push(&mut self, v: u32) -> Result<(), H2SettingsError> {
        self.enable_push = match v {
            0 => false,
            1 => true,
            _ => return Err(H2SettingsError::InvalidEnablePush),
        };
        Ok(())
    }
    /// Set SETTINGS_MAX_CONCURRENT_STREAMS (0x03)
    pub fn set_max_concurrent_streams(&mut self, v: u32) -> Result<(), H2SettingsError> {
        self.max_concurrent_streams = v;
        Ok(())
    }
    /// Set SETTINGS_INITIAL_WINDOW_SIZE (0x04)
    pub fn set_initial_window_size(&mut self, v: u32) -> Result<(), H2SettingsError> {
        self.initial_window_size = match v {
            ..2147483647 => v,
            _ => return Err(H2SettingsError::FlowControlError),
        };
        Ok(())
    }
    /// Set SETTINGS_MAX_FRAME_SIZE (0x05)
    pub fn set_max_frame_size(&mut self, v: u32) -> Result<(), H2SettingsError> {
        self.max_frame_size = match v {
            16_384..16_777_215 => v,
            _ => return Err(H2SettingsError::ProtocolError),
        };
        Ok(())
    }
    /// Set SETTINGS_MAX_HEADER_LIST_SIZE (0x06)
    pub fn set_max_header_list_size(&mut self, v: u32) -> Result<(), H2SettingsError> {
        self.max_header_list_size = v;
        Ok(())
    }
    /// Construct H2 Settings from settings frame payload
    pub fn from_payload(payload: &[u8]) -> Result<Self, H2SettingsError> {
        let mut settings = H2Settings::default();
        if payload.len() % 6 != 0 {
            return Err(H2SettingsError::InvalidPayload);
        }
        for chnk in payload.chunks(6) {
            let val = u32::from_be_bytes([chnk[2], chnk[3], chnk[4], chnk[5]]);
            match chnk[1] {
                0x01 => settings.set_header_table_size(val)?,
                0x02 => settings.set_enable_push(val)?,
                0x03 => settings.set_max_concurrent_streams(val)?,
                0x04 => settings.set_initial_window_size(val)?,
                0x05 => settings.set_max_frame_size(val)?,
                0x06 => settings.set_max_header_list_size(val)?,
                _ => {}
            }
        }
        Ok(settings)
    }
}

impl H2Settings {
    /// Send the settings as a frame out using FrameReceiver per RFC 9113 s. 6.5.1. SETTINGS Format
    pub fn frame_out<R: FrameReceiver + SettingsFrameBuilder>(
        &self,
        r: &mut R,
    ) -> Result<(), BuilderError> {
        let mut frame_out: [u8; 60] = [0; 60];
        let mut filler = FrameFiller::<60>::new();

        // Size of the completed frame u24
        filler.try_skip_only(3)?;

        // SETTINGS
        filler.try_fill_with(&mut frame_out, &[0x04])?;

        // ACK flag
        let ack_flag = r.ack_flag();
        let ack_byte = match ack_flag {
            true => 1u8,
            false => 0u8,
        };
        filler.try_fill_with(&mut frame_out, &[ack_byte])?;

        // StreamID (type limited to u31)
        let stream_id = r.stream_id().as_u32();
        let stream_id_bytes = stream_id.to_be_bytes();
        filler.try_fill_with(&mut frame_out, &stream_id_bytes)?;

        let frame_header_length = filler.cur_as_u32();

        let enable_push_u32: u32 = match self.enable_push {
            false => 0,
            true => 1,
        };

        // Settings Id 2 bytes, Value 4 bytes be-u32
        let settings = [
            (0x01, self.header_table_size),
            (0x02, enable_push_u32),
            (0x03, self.max_concurrent_streams),
            (0x04, self.initial_window_size),
            (0x05, self.max_frame_size),
            (0x06, self.max_header_list_size),
        ];

        for (setting_id_short, setting_val) in settings {
            let val_bytes = setting_val.to_be_bytes();
            filler.try_fill_with(&mut frame_out, &[0, setting_id_short])?;
            filler.try_fill_with(&mut frame_out, &val_bytes)?;
        }
        let filled_len = filler.cur_as_u32() - frame_header_length;
        let filled_len_b32 = filled_len.to_be_bytes();
        frame_out[0] = filled_len_b32[1];
        frame_out[1] = filled_len_b32[2];
        frame_out[2] = filled_len_b32[3];

        r.frame(&frame_out[..filler.cur_as_usize()]);

        Ok(())
    }
}

mod test {

    use super::*;
    use crate::StreamId;

    #[rustfmt::skip]
    #[allow(dead_code)]
    const _CURL_EXPECTED: &[u8] = &[
        0, 0, 18,           // 18 Length (Settings)
        4,                  // Settings Type
        0,                  // ACK
        0, 0, 0, 0,         // StreamId
        0, 3, 0, 0, 0, 100, // Max concurrent streams 6..
        0, 4, 0, 1, 0, 0,   // Initial window size    12..
        0, 2, 0, 0, 0, 0,   // Enable push            ..18
    ];

    #[rustfmt::skip]
    #[allow(dead_code)]
    const EXPECTED_DEFAULT: &[u8] = &[
        0, 0, 36,
        4,
        0,
        0, 0, 0, 0,
        0, 1, 0, 0, 16, 0,
        0, 2, 0, 0, 0, 1,
        0, 3, 0, 0, 0, 100,
        0, 4, 0, 0, 255, 255,
        0, 5, 0, 0, 64, 0,
        0, 6, 0, 0, 0, 0        
    ];

    #[derive(Debug, Default)]
    #[allow(dead_code)] // insta assert's debug snapshot
    struct TestFramer {
        b: Vec<u8>,
    }

    impl SettingsFrameBuilder for TestFramer {
        fn stream_id(&mut self) -> StreamId {
            StreamId::unchecked_new(0)
        }
        fn ack_flag(&mut self) -> bool {
            false
        }
    }

    impl FrameReceiver for TestFramer {
        fn frame(&mut self, frame: &[u8]) {
            self.b.extend(frame);
        }
    }

    #[test]
    fn settings_builder_test_default() {
        let mut framer = TestFramer::default();
        let settings = H2Settings::default();
        match settings.frame_out(&mut framer) {
            Ok(()) => {}
            Err(e) => panic!("No frame out: {:?}", e),
        }
        assert_eq!(EXPECTED_DEFAULT, framer.b);
    }
}
