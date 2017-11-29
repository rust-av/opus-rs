use ffi::opus::*;
use common::*;

pub struct Encoder {
    enc: *mut OpusMSEncoder,
    channels: usize,
}

unsafe impl Send for Encoder {} // TODO: Make sure it cannot be abused

#[repr(i32)]
pub enum Application {
    Voip = OPUS_APPLICATION_VOIP as i32,
    Audio = OPUS_APPLICATION_AUDIO as i32,
    LowDelay = OPUS_APPLICATION_RESTRICTED_LOWDELAY as i32,
}

impl Encoder {
    pub fn create(
        sample_rate: usize,
        channels: usize,
        streams: usize,
        coupled_streams: usize,
        mapping: &[u8],
        application: Application,
    ) -> Result<Encoder, ErrorCode> {
        let mut err = 0;
        let enc = unsafe {
            opus_multistream_encoder_create(
                sample_rate as i32,
                channels as i32,
                streams as i32,
                coupled_streams as i32,
                mapping.as_ptr(),
                application as i32,
                &mut err,
            )
        };

        if err < 0 {
            Err(err.into())
        } else {
            Ok(Encoder {
                enc: enc,
                channels: channels,
            })
        }
    }

    pub fn encode<'a, I>(&mut self, input: I, output: &mut [u8]) -> Result<(), ErrorCode>
    where
        I: Into<AudioBuffer<'a>>,
    {
        let ret = match input.into() {
            AudioBuffer::F32(v) => unsafe {
                opus_multistream_encode_float(
                    self.enc,
                    v.as_ptr(),
                    (v.len() / self.channels) as i32,
                    output.as_mut_ptr(),
                    output.len() as i32,
                )
            },
            AudioBuffer::I16(v) => unsafe {
                opus_multistream_encode(
                    self.enc,
                    v.as_ptr(),
                    (v.len() / self.channels) as i32,
                    output.as_mut_ptr(),
                    output.len() as i32,
                )
            },
        };

        if ret < 0 { Err(ret.into()) } else { Ok(()) }
    }

    pub fn set_option(&mut self, key: u32, val: u32) -> Result<(), ErrorCode> {
        let ret = match key {
            OPUS_SET_APPLICATION_REQUEST |
            OPUS_SET_BITRATE_REQUEST |
            OPUS_SET_MAX_BANDWIDTH_REQUEST |
            OPUS_SET_VBR_REQUEST |
            OPUS_SET_BANDWIDTH_REQUEST |
            OPUS_SET_COMPLEXITY_REQUEST |
            OPUS_SET_INBAND_FEC_REQUEST |
            OPUS_SET_PACKET_LOSS_PERC_REQUEST |
            OPUS_SET_DTX_REQUEST |
            OPUS_SET_VBR_CONSTRAINT_REQUEST |
            OPUS_SET_FORCE_CHANNELS_REQUEST |
            OPUS_SET_SIGNAL_REQUEST |
            OPUS_SET_GAIN_REQUEST |
            OPUS_SET_LSB_DEPTH_REQUEST |
            OPUS_SET_EXPERT_FRAME_DURATION_REQUEST |
            OPUS_SET_PREDICTION_DISABLED_REQUEST |
            OPUS_SET_PHASE_INVERSION_DISABLED_REQUEST => unsafe {
                opus_multistream_encoder_ctl(self.enc, key as i32, val)
            },
            _ => unimplemented!(),
        };

        if ret < 0 { Err(ret.into()) } else { Ok(()) }
    }
    pub fn get_option(&mut self, key: u32) -> Result<i32, ErrorCode> {
        let mut val: i32 = 0;
        let ret = match key {
            OPUS_GET_LOOKAHEAD_REQUEST => unsafe {
                opus_multistream_encoder_ctl(self.enc, key as i32, &mut val as *mut i32)
            },
            _ => unimplemented!(),
        };

        if ret < 0 { Err(ret.into()) } else { Ok(val) }
    }

    pub fn reset(&mut self) {
        let _ = unsafe { opus_multistream_encoder_ctl(self.enc, OPUS_RESET_STATE as i32) };
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe { opus_multistream_encoder_destroy(self.enc) };
    }
}
