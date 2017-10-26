use ffi::opus::*;
use common::*;

use std::ptr;

pub struct Decoder {
    dec: *mut OpusMSDecoder,
    channels: usize,
}

impl Decoder {
    pub fn create(
        sample_rate: usize,
        channels: usize,
        streams: usize,
        coupled_streams: usize,
        mapping: &[u8],
    ) -> Result<Decoder, ErrorCode> {
        let mut err = 0;
        let dec = unsafe {
            opus_multistream_decoder_create(
                sample_rate as i32,
                channels as i32,
                streams as i32,
                coupled_streams as i32,
                mapping.as_ptr(),
                &mut err,
            )
        };

        if err < 0 {
            Err(err.into())
        } else {
            Ok(Decoder {
                dec: dec,
                channels: channels,
            })
        }
    }

    pub fn decode<'a, I, O>(&mut self, input: I, out: O, decode_fec: bool) -> Result<(), ErrorCode>
    where
        I: Into<Option<&'a [u8]>>,
        O: Into<AudioBufferMut<'a>>,
    {
        let (data, len) = input.into().map_or(
            (ptr::null(), 0),
            |v| (v.as_ptr(), v.len()),
        );

        let ret = match out.into() {
            AudioBufferMut::F32(v) => unsafe {
                opus_multistream_decode_float(
                    self.dec,
                    data,
                    len as i32,
                    v.as_mut_ptr(),
                    (v.len() / self.channels) as i32,
                    decode_fec as i32,
                )
            },
            AudioBufferMut::I16(v) => unsafe {
                opus_multistream_decode(
                    self.dec,
                    data,
                    len as i32,
                    v.as_mut_ptr(),
                    (v.len() / self.channels) as i32,
                    decode_fec as i32,
                )
            },
        };

        if ret < 0 { Err(ret.into()) } else { Ok(()) }
    }

    pub fn set_option(&mut self, key: u32, val: i32) -> Result<(), ErrorCode>
    {
        let ret = match key {
            OPUS_SET_GAIN_REQUEST => unsafe {
                opus_multistream_decoder_ctl(self.dec, key as i32, val)
            },
            _ => unimplemented!(),
        };

        if ret < 0 { Err(ret.into()) } else { Ok(()) }
    }

    pub fn reset(&mut self) {
        let _ = unsafe { opus_multistream_decoder_ctl(self.dec, OPUS_RESET_STATE as i32) };
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe { opus_multistream_decoder_destroy(self.dec) }
    }
}


#[cfg(feature="codec-trait")]
mod decoder_trait {
    use super::Decoder as OpusDecoder;
    use codec::decoder::*;
    use codec::error::*;
    use bitstream::byteread::get_i16l;
    use data::packet::Packet;
    use data::frame::*;
    use data::audiosample::formats::S16;
    use data::audiosample::ChannelMap;
    use std::rc::Rc;
    use std::collections::VecDeque;
    use ffi::opus::OPUS_SET_GAIN_REQUEST;

    struct Des {
        descr: Descr,
    }

    struct Dec {
        dec: Option<OpusDecoder>,
        extradata: Option<Vec<u8>>,
        pending: VecDeque<Frame>,
        info: AudioInfo,
    }

    impl Dec {
        fn new() -> Self {
            Dec { dec: None,
                extradata: None,
                pending: VecDeque::with_capacity(1),
                info: AudioInfo {
                    samples: 960 * 6,
                    rate: 48000,
                    map: ChannelMap::new(),
                    format: Rc::new(S16)
                }
            }
        }
    }

    impl Descriptor for Des {
        fn create(&self) -> Box<Decoder> {
            Box::new(Dec::new())
        }

        fn describe<'a>(&'a self) -> &'a Descr {
            &self.descr
        }
    }

    const OPUS_HEAD_SIZE: usize = 19;

    impl Decoder for Dec {
        fn set_extradata(&mut self, extra: &[u8]) {
            self.extradata = Some(Vec::from(extra));
        }
        fn send_packet(&mut self, pkt: &Packet) -> Result<()> {
            let mut f = new_default_frame(self.info.clone(), None);

            let ret;
            {
                let buf : &mut [i16] = f.buf.as_mut_slice(0).unwrap();
                ret = self.dec.as_mut().unwrap()
                    .decode(pkt.data.as_slice(), buf, false)
                    .map_err(|_e| ErrorKind::InvalidData.into());
            }

            match ret {
                Ok(_) => { self.pending.push_back(f); Ok(()) },
                Err(e) => Err(e)
            }
        }
        fn receive_frame(&mut self) -> Result<Frame> {
            self.pending.pop_front().ok_or(ErrorKind::MoreDataNeeded.into())
        }
        fn reset(&mut self) -> Result<()> {
            let channels;
            let sample_rate = 48000;
            let mut gain_db = 0;
            let mut streams = 1;
            let mut coupled_streams = 0;
            let mut mapping : &[u8] = &[0u8, 1u8];
            let mut channel_map = false;

            if let Some(ref extradata) = self.extradata {
                channels = *extradata.get(9).unwrap_or(&2) as usize;

                if extradata.len() >= OPUS_HEAD_SIZE {
                    gain_db = get_i16l(&extradata[16..17]);
                    channel_map = extradata[18] != 0;
                }
                if extradata.len() >= OPUS_HEAD_SIZE + 2 + channels {
                    streams = extradata[OPUS_HEAD_SIZE] as usize;
                    coupled_streams = extradata[OPUS_HEAD_SIZE + 1] as usize;
                    if streams + coupled_streams != channels {
                        unimplemented!()
                    }
                    mapping = &extradata[OPUS_HEAD_SIZE + 2 ..]
                } else {
                    if channels > 2 || channel_map {
                        return Err(ErrorKind::InvalidConfiguration.into());
                    }
                    if channels > 1 {
                        coupled_streams = 1;
                    }
                }
            } else {
                return Err(ErrorKind::ConfigurationIncomplete.into());
            }

            if channels > 2 {
                unimplemented!() // TODO: Support properly channel mapping
            } else {
                self.info.map = ChannelMap::default_map(channels);
            }

            match OpusDecoder::create(sample_rate, channels, streams, coupled_streams, mapping) {
                Ok(mut d) => {
                    let _ = d.set_option(OPUS_SET_GAIN_REQUEST, gain_db as i32);
                    self.dec = Some(d);
                    Ok(())
                },
                Err(_) => Err(ErrorKind::InvalidConfiguration.into())
            }
        }
    }

    pub const OPUS_DESCR: &Descriptor = &Des {
        descr: Descr {
            codec: "opus",
            name: "libopus",
            desc: "libopus decoder",
            mime: "audio/OPUS",
        },
    };
}

#[cfg(feature="codec-trait")]
pub use self::decoder_trait::OPUS_DESCR;
