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

    pub fn set_option(&mut self, key: u32, val: i32) -> Result<(), ErrorCode> {
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
