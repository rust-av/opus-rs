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

impl Application {
    pub fn from_str<'a>(s : &'a str) -> Option<Self> {
        use self::Application::*;
        match s {
            "voip" => Some(Voip),
            "audio" => Some(Audio),
            "lowdelay" => Some(LowDelay),
            _ => None
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

#[cfg(feature="codec-trait")]
mod encoder_trait {
    use super::Encoder as OpusEncoder;
    use super::Application;
    // use std::rc::Rc;
    use codec::encoder::*;
    use codec::error::*;
    // use data::audiosample::Soniton;
    // use data::audiosample::formats::S16;
    use data::value::Value;
    use data::frame::{ArcFrame, MediaKind, FrameBufferConv};
    use data::packet::Packet;
    use data::rational::Rational64;
    use std::collections::VecDeque;

    struct Des {
        descr: Descr,
    }

    struct Cfg {
        channels: usize,
        streams: usize,
        coupled_streams: usize,
        mapping: Vec<u8>,
        application: Option<Application>,
    }

    impl Cfg {
        fn is_valid(&self) -> bool {
            self.channels > 0 && self.streams + self.coupled_streams == self.channels && self.mapping.len() == self.channels
        }
    }

    struct Enc {
        enc: Option<OpusEncoder>,
        pending: VecDeque<Packet>,
        frame_size: usize,
        cfg: Cfg,
    }

    impl Descriptor for Des {
        fn create(&self) -> Box<Encoder> {
            Box::new(Enc {
                enc: None,
                pending: VecDeque::new(),
                frame_size: 0,
                cfg: Cfg { channels: 0, streams: 0, coupled_streams: 0, mapping: Vec::new(), application: None }
            })
        }

        fn describe<'a>(&'a self) -> &'a Descr {
            &self.descr
        }
    }

    // Values copied from libopusenc.c
    // A packet may contain up to 3 frames, each of 1275 bytes max.
    // The packet header may be up to 7 bytes long.

    const MAX_HEADER_SIZE : usize = 7;
    const MAX_FRAME_SIZE : usize = 1275;
    const MAX_FRAMES : usize = 3;

    impl Encoder for Enc {
        fn configure(&mut self) -> Result<()> {
            if self.enc.is_none() {
                if self.cfg.is_valid() {
                    unimplemented!()
                } else {
                    unimplemented!()
                }
            } else {
                unimplemented!()
            }
        }

        fn get_extradata(&self) -> Option<Vec<u8>> {
            unimplemented!()
        }

        fn send_frame(&mut self, frame: &ArcFrame) -> Result<()> {
            let enc = self.enc.as_mut().unwrap();
            let pending = &mut self.pending;
            if let MediaKind::Audio(ref info) = frame.kind {
                let channels = info.map.len();
                let input_size = info.samples * channels;
                let input : &[i16] = frame.buf.as_slice(0).unwrap();
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
                        Ok(_) => {
                            let duration = (Rational64::new(len as i64 / channels as i64, 48000) / frame.t.timebase.unwrap()).to_integer();
                            pkt.t.pts = Some(pts);
                            pkt.t.dts = Some(pts);
                            pkt.t.duration = Some(duration as u64);
                            pts += duration;
                            pending.push_back(pkt);
                        },
                        Err(e) => {
                            match e {
                                _ => unimplemented!()
                            }
                        }
                    }
                }
                Ok(())
            } else {
                unimplemented!() // TODO mark it unreachable?
            }
        }

        fn receive_packet(&mut self) -> Result<Packet> {
            self.pending.pop_front().ok_or(ErrorKind::MoreDataNeeded.into())
        }

        fn set_option<'a>(&mut self, key: &str, val: Value<'a>) -> Result<()> {
            match (key, val) {
                // ("format", Value::Formaton(f)) => self.format = Some(f),
                // ("mapping", Value::ChannelMap(map) => self.cfg.map = map::to_vec()
                ("channels", Value::U64(v)) => self.cfg.channels = v as usize,
                ("streams", Value::U64(v)) => self.cfg.streams = v as usize,
                ("coupled_streams", Value::U64(v)) => self.cfg.coupled_streams = v as usize,
                ("application", Value::Str(s)) => {
                    if let Some(a) = Application::from_str(s) {
                        self.cfg.application = Some(a);
                    } else {
                        return Err(ErrorKind::InvalidData.into());
                    }
                },
                _ => unimplemented!(),
            }

            Ok(())
        }
        fn flush(&mut self) -> Result<()> {
            unimplemented!()
        }
    }

    pub const OPUS_DESCR: &Descriptor = &Des {
        descr: Descr {
            codec: "opus",
            name: "libopus",
            desc: "libopus encoder",
            mime: "audio/OPUS",
        },
    };
}

#[cfg(feature="codec-trait")]
pub use self::encoder_trait::OPUS_DESCR;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn init() {
    }
}
