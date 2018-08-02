extern crate opus_sys;
#[macro_use]
extern crate structopt;
extern crate av_bitstream as bitstream;

use opus_sys::opus::*;

use structopt::StructOpt;

use std::mem;

use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

use bitstream::bytewrite::put_i32b;

#[derive(Debug, StructOpt)]
#[structopt(name = "encoder", about = "Opus encoding example")]
struct EncodingOpts {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output file
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    /// Sampling rate, in Hz
    #[structopt(default_value = "48000")]
    sampling_rate: i32,
    /// Channels, either 1 or 2
    #[structopt(default_value = "1")]
    channels: i32,
    /// Bitrate
    #[structopt(default_value = "16000")]
    bits_per_second: i32,
    /// Number of seconds to encode
    #[structopt(default_value = "10")]
    seconds: i32,
}

trait Encode {
    fn get_encoder(&self) -> Option<*mut OpusEncoder>;
}

impl Encode for EncodingOpts {
    fn get_encoder(&self) -> Option<*mut OpusEncoder> {
        let mut err = unsafe { mem::uninitialized() };
        let enc = unsafe { opus_encoder_create(self.sampling_rate, self.channels, OPUS_APPLICATION_AUDIO as i32, &mut err) };

        if err != OPUS_OK as i32 {
            None
        } else {
            Some(enc)
        }
    }
}

use std::slice;

fn main() {
    let enc_opt = EncodingOpts::from_args();

    println!("{:?}", enc_opt);

    let enc = enc_opt.get_encoder().unwrap();

    unsafe {
        opus_encoder_ctl(enc, OPUS_SET_APPLICATION_REQUEST as i32, OPUS_APPLICATION_AUDIO as i32);
        opus_encoder_ctl(enc, OPUS_SET_BITRATE_REQUEST as i32, enc_opt.bits_per_second);
        opus_encoder_ctl(enc, OPUS_SET_BANDWIDTH_REQUEST as i32, OPUS_BANDWIDTH_WIDEBAND);
        opus_encoder_ctl(enc, OPUS_SET_COMPLEXITY_REQUEST as i32, 10);
        opus_encoder_ctl(enc, OPUS_SET_VBR_REQUEST as i32, 0);
        opus_encoder_ctl(enc, OPUS_SET_VBR_CONSTRAINT_REQUEST as i32, 0);
        opus_encoder_ctl(enc, OPUS_SET_PACKET_LOSS_PERC_REQUEST as i32, 0);
    }

    let mut in_f = File::open(enc_opt.input).unwrap();
    let mut out_f = File::create(enc_opt.output).unwrap();

    let frame_size = 2880;
    let total_bytes = (enc_opt.channels * enc_opt.seconds * enc_opt.sampling_rate * 2) as usize;
    let max_packet = 1500;
    let mut processed_bytes = 0;
    let mut buf = Vec::with_capacity(frame_size * 2);
    let mut out_buf = Vec::with_capacity(max_packet);

    buf.resize(frame_size * 2, 0u8);
    out_buf.resize(max_packet, 0u8);

    while processed_bytes < total_bytes {
        in_f.read_exact(&mut buf).unwrap();

        let samples = unsafe {
            slice::from_raw_parts(mem::transmute(buf.as_ptr()), frame_size)
        };

        processed_bytes += frame_size * 2;

        let ret = unsafe {
            opus_encode(enc,
                        samples.as_ptr(), samples.len() as i32,
                        out_buf.as_mut_ptr(), out_buf.len() as i32)
        };

        if ret > 0 {
            let mut b = [0u8; 4];

            // Write the packet size
            put_i32b(&mut b, ret);
            out_f.write_all(&b).unwrap();

            // Write the encoder ec final state
            let mut val = 0i32;
            unsafe { opus_encoder_ctl(enc, OPUS_GET_FINAL_RANGE_REQUEST as i32, &mut val) };
            put_i32b(&mut b, val);
            out_f.write_all(&b).unwrap();

            // Write the actual packet
            out_f.write_all(&out_buf[..ret as usize]).unwrap();
        } else {
            panic!("Cannot encode");
        }
    }
}
