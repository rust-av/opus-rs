use crate::common::*;
use crate::ffi::*;
use std::str::FromStr;

pub struct Encoder {
    enc: *mut OpusMSEncoder,
    channels: usize,
}

mod constants {
    pub use ffi::OPUS_SET_APPLICATION_REQUEST;
    pub use ffi::OPUS_SET_BANDWIDTH_REQUEST;
    pub use ffi::OPUS_SET_BITRATE_REQUEST;
    pub use ffi::OPUS_SET_COMPLEXITY_REQUEST;
    pub use ffi::OPUS_SET_DTX_REQUEST;
    pub use ffi::OPUS_SET_EXPERT_FRAME_DURATION_REQUEST;
    pub use ffi::OPUS_SET_FORCE_CHANNELS_REQUEST;
    pub use ffi::OPUS_SET_GAIN_REQUEST;
    pub use ffi::OPUS_SET_INBAND_FEC_REQUEST;
    pub use ffi::OPUS_SET_LSB_DEPTH_REQUEST;
    pub use ffi::OPUS_SET_MAX_BANDWIDTH_REQUEST;
    pub use ffi::OPUS_SET_PACKET_LOSS_PERC_REQUEST;
    pub use ffi::OPUS_SET_PREDICTION_DISABLED_REQUEST;
    pub use ffi::OPUS_SET_SIGNAL_REQUEST;
    pub use ffi::OPUS_SET_VBR_CONSTRAINT_REQUEST;
    pub use ffi::OPUS_SET_VBR_REQUEST;

    pub use ffi::OPUS_GET_FINAL_RANGE_REQUEST;
    pub use ffi::OPUS_GET_LOOKAHEAD_REQUEST;

    pub use ffi::OPUS_BANDWIDTH_FULLBAND;
    pub use ffi::OPUS_BANDWIDTH_MEDIUMBAND;
    pub use ffi::OPUS_BANDWIDTH_NARROWBAND;
    pub use ffi::OPUS_BANDWIDTH_SUPERWIDEBAND;
    pub use ffi::OPUS_BANDWIDTH_WIDEBAND;
    pub use ffi::OPUS_FRAMESIZE_100_MS;
    pub use ffi::OPUS_FRAMESIZE_10_MS;
    pub use ffi::OPUS_FRAMESIZE_120_MS;
    pub use ffi::OPUS_FRAMESIZE_20_MS;
    pub use ffi::OPUS_FRAMESIZE_2_5_MS;
    pub use ffi::OPUS_FRAMESIZE_40_MS;
    pub use ffi::OPUS_FRAMESIZE_5_MS;
    pub use ffi::OPUS_FRAMESIZE_60_MS;
    pub use ffi::OPUS_FRAMESIZE_80_MS;
    pub use ffi::OPUS_FRAMESIZE_ARG;
}

pub use self::constants::*;

unsafe impl Send for Encoder {} // TODO: Make sure it cannot be abused

#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum Application {
    Voip = OPUS_APPLICATION_VOIP as i32,
    Audio = OPUS_APPLICATION_AUDIO as i32,
    LowDelay = OPUS_APPLICATION_RESTRICTED_LOWDELAY as i32,
}

impl FromStr for Application {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Application::*;
        match s {
            "voip" => Ok(Voip),
            "audio" => Ok(Audio),
            "lowdelay" => Ok(LowDelay),
            _ => Err(()),
        }
    }
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
            Ok(Encoder { enc, channels })
        }
    }

    pub fn encode<'a, I>(&mut self, input: I, output: &mut [u8]) -> Result<usize, ErrorCode>
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

        if ret < 0 {
            Err(ret.into())
        } else {
            Ok(ret as usize)
        }
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
            OPUS_SET_PREDICTION_DISABLED_REQUEST /* |
            OPUS_SET_PHASE_INVERSION_DISABLED_REQUEST */ => unsafe {
                opus_multistream_encoder_ctl(self.enc, key as i32, val)
            },
            _ => unimplemented!(),
        };

        if ret < 0 {
            Err(ret.into())
        } else {
            Ok(())
        }
    }
    pub fn get_option(&self, key: u32) -> Result<i32, ErrorCode> {
        let mut val: i32 = 0;
        let ret = match key {
            OPUS_GET_LOOKAHEAD_REQUEST | OPUS_GET_FINAL_RANGE_REQUEST => unsafe {
                opus_multistream_encoder_ctl(self.enc, key as i32, &mut val as *mut i32)
            },
            _ => unimplemented!(),
        };

        if ret < 0 {
            Err(ret.into())
        } else {
            Ok(val)
        }
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

#[cfg(feature = "codec-trait")]
mod encoder_trait {
    use super::constants::*;
    use super::Application;
    use super::Encoder as OpusEncoder;
    // use std::rc::Rc;
    use codec::encoder::*;
    use codec::error::*;
    use data::audiosample::formats::S16;
    use data::audiosample::ChannelMap;
    use data::frame::{ArcFrame, FrameBufferConv, MediaKind};
    use data::packet::Packet;
    use data::params::CodecParams;
    use data::rational::Rational64;
    use data::value::Value;
    use std::collections::VecDeque;

    pub struct Des {
        descr: Descr,
    }

    struct Cfg {
        channels: usize,
        streams: usize,
        coupled_streams: usize,
        mapping: Vec<u8>,
        application: Application,
        bitrate: usize,
    }

    impl Cfg {
        fn is_valid(&self) -> bool {
            self.channels > 0
                && self.streams + self.coupled_streams == self.channels
                && self.mapping.len() == self.channels
        }
    }

    pub struct Enc {
        enc: Option<OpusEncoder>,
        pending: VecDeque<Packet>,
        frame_size: usize,
        delay: usize,
        cfg: Cfg,
        flushing: bool,
    }

    impl Descriptor for Des {
        type OutputEncoder = Enc;

        fn create(&self) -> Self::OutputEncoder {
            Enc {
                enc: None,
                pending: VecDeque::new(),
                frame_size: 960,
                delay: 0,
                cfg: Cfg {
                    channels: 0,
                    streams: 0,
                    coupled_streams: 0,
                    mapping: vec![0, 1],
                    application: Application::Audio,
                    bitrate: 16000,
                },
                flushing: false,
            }
        }

        fn describe(&self) -> &Descr {
            &self.descr
        }
    }

    // Values copied from libopusenc.c
    // A packet may contain up to 3 frames, each of 1275 bytes max.
    // The packet header may be up to 7 bytes long.

    const MAX_HEADER_SIZE: usize = 7;
    const MAX_FRAME_SIZE: usize = 1275;
    const MAX_FRAMES: usize = 3;

    /// 80ms in samples
    const CONVERGENCE_WINDOW: usize = 3840;

    impl Encoder for Enc {
        fn configure(&mut self) -> Result<()> {
            if self.enc.is_none() {
                if self.cfg.is_valid() {
                    let mut enc = OpusEncoder::create(
                        48000, // TODO
                        self.cfg.channels,
                        self.cfg.streams,
                        self.cfg.coupled_streams,
                        &self.cfg.mapping,
                        self.cfg.application,
                    )
                    .map_err(|_e| unimplemented!())?;
                    enc.set_option(OPUS_SET_BITRATE_REQUEST, self.cfg.bitrate as u32)
                        .unwrap();
                    enc.set_option(OPUS_SET_BANDWIDTH_REQUEST, OPUS_BANDWIDTH_WIDEBAND)
                        .unwrap();
                    enc.set_option(OPUS_SET_COMPLEXITY_REQUEST, 10).unwrap();
                    enc.set_option(OPUS_SET_VBR_REQUEST, 0).unwrap();
                    enc.set_option(OPUS_SET_VBR_CONSTRAINT_REQUEST, 0).unwrap();
                    enc.set_option(OPUS_SET_PACKET_LOSS_PERC_REQUEST, 0)
                        .unwrap();

                    self.delay = enc.get_option(OPUS_GET_LOOKAHEAD_REQUEST).unwrap() as usize;
                    self.enc = Some(enc);
                    Ok(())
                } else {
                    unimplemented!()
                }
            } else {
                unimplemented!()
            }
        }
        // TODO: support multichannel
        fn get_extradata(&self) -> Option<Vec<u8>> {
            use bitstream::bytewrite::*;
            if self.cfg.channels > 2 {
                unimplemented!();
            }

            let mut buf = b"OpusHead".to_vec();

            buf.resize(19, 0);

            buf[8] = 1;
            buf[9] = self.cfg.channels as u8;
            put_i16l(&mut buf[10..12], self.delay as i16);
            put_i32l(&mut buf[12..16], 48000); // TODO
            put_i16l(&mut buf[16..18], 0);
            buf[18] = 0;

            Some(buf)
        }

        fn send_frame(&mut self, frame: &ArcFrame) -> Result<()> {
            let enc = self.enc.as_mut().unwrap();
            let pending = &mut self.pending;
            if let MediaKind::Audio(ref info) = frame.kind {
                let channels = info.map.len();
                let input_size = info.samples * channels;
                let input: &[i16] = frame.buf.as_slice(0).unwrap();
                let data_size = MAX_HEADER_SIZE + MAX_FRAMES * MAX_FRAME_SIZE;
                let chunk_size = self.frame_size * channels;
                let mut buf = Vec::with_capacity(chunk_size);
                let mut pts = frame.t.pts.unwrap();

                for chunk in input[..input_size].chunks(chunk_size) {
                    let len = chunk.len();
                    let mut pkt = Packet::with_capacity(data_size);

                    pkt.data.resize(data_size, 0); // TODO is it needed?

                    let input_data = if len < chunk_size {
                        buf.clear();
                        buf.extend_from_slice(chunk);
                        buf.as_slice()
                    } else {
                        chunk
                    };

                    match enc.encode(input_data, pkt.data.as_mut_slice()) {
                        Ok(len) => {
                            let duration = (Rational64::new(len as i64 / channels as i64, 48000)
                                / frame.t.timebase.unwrap())
                            .to_integer();
                            pkt.t.pts = Some(pts);
                            pkt.t.dts = Some(pts);
                            pkt.t.duration = Some(duration as u64);
                            pts += duration;
                            pkt.data.truncate(len);
                            pending.push_back(pkt);
                        }
                        Err(e) => match e {
                            _ => unimplemented!(),
                        },
                    }
                }
                Ok(())
            } else {
                unimplemented!() // TODO mark it unreachable?
            }
        }

        fn receive_packet(&mut self) -> Result<Packet> {
            self.pending.pop_front().ok_or(Error::MoreDataNeeded)
        }

        fn set_option<'a>(&mut self, key: &str, val: Value<'a>) -> Result<()> {
            match (key, val) {
                // ("format", Value::Formaton(f)) => self.format = Some(f),
                // ("mapping", Value::ChannelMap(map) => self.cfg.map = map::to_vec()
                ("channels", Value::U64(v)) => self.cfg.channels = v as usize,
                ("streams", Value::U64(v)) => self.cfg.streams = v as usize,
                ("coupled_streams", Value::U64(v)) => self.cfg.coupled_streams = v as usize,
                ("application", Value::Str(s)) => {
                    if let Ok(a) = s.parse() {
                        self.cfg.application = a;
                    } else {
                        return Err(Error::InvalidData);
                    }
                }
                _ => return Err(Error::Unsupported("Unsupported option".to_owned())),
            }

            Ok(())
        }

        fn set_params(&mut self, params: &CodecParams) -> Result<()> {
            use data::params::*;
            if let Some(MediaKind::Audio(ref info)) = params.kind {
                if let Some(ref map) = info.map {
                    if map.len() > 2 {
                        unimplemented!()
                    } else {
                        self.cfg.channels = map.len();
                        self.cfg.coupled_streams = self.cfg.channels - 1;
                        self.cfg.streams = 1;
                        self.cfg.mapping = if map.len() > 1 { vec![0, 1] } else { vec![0] };
                    }
                }
            }
            Ok(())
        }

        // TODO: guard against calling it before configure()
        // is issued.
        fn get_params(&self) -> Result<CodecParams> {
            use data::params::*;
            use std::sync::Arc;
            Ok(CodecParams {
                kind: Some(MediaKind::Audio(AudioInfo {
                    rate: 48000,
                    map: Some(ChannelMap::default_map(2)),
                    format: Some(Arc::new(S16)),
                })),
                codec_id: Some("opus".to_owned()),
                extradata: self.get_extradata(),
                bit_rate: 0, // TODO: expose the information
                convergence_window: CONVERGENCE_WINDOW,
                delay: self.delay,
            })
        }

        fn flush(&mut self) -> Result<()> {
            // unimplemented!()
            self.flushing = true;
            Ok(())
        }
    }

    pub const OPUS_DESCR: &Des = &Des {
        descr: Descr {
            codec: "opus",
            name: "libopus",
            desc: "libopus encoder",
            mime: "audio/OPUS",
        },
    };
}

#[cfg(feature = "codec-trait")]
pub use self::encoder_trait::OPUS_DESCR;
