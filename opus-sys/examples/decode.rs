extern crate opus_sys;
extern crate structopt;
extern crate av_bitstream as bitstream;

use opus_sys::*;

use structopt::StructOpt;

use std::mem::MaybeUninit;

use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

use bitstream::byteread::get_i32b;

#[derive(Debug, StructOpt)]
#[structopt(name = "decoder", about = "Opus decoding example")]
struct DecodingOpts {
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
    /// Number of seconds to decode
    #[structopt(default_value = "10")]
    seconds: i32,
}

trait Decode {
    fn get_decoder(&self) -> Option<*mut OpusDecoder>;
}

impl Decode for DecodingOpts {
    fn get_decoder(&self) -> Option<*mut OpusDecoder> {
        let mut err = MaybeUninit::uninit();
        let dec = unsafe { opus_decoder_create(self.sampling_rate, self.channels, err.as_mut_ptr()) };
        let err = unsafe { err.assume_init() };

        if err != OPUS_OK as i32 {
            None
        } else {
            Some(dec)
        }
    }
}

use std::slice;

fn main() {
    let dec_opt = DecodingOpts::from_args();

    let dec = dec_opt.get_decoder().unwrap();

    let mut in_f = File::open(dec_opt.input).unwrap();
    let mut out_f = File::create(dec_opt.output).unwrap();

    let max_packet = 1500;
    let max_frame = 48000 * 2;
    let max_frame_samples = max_frame * dec_opt.channels as usize;

    let mut pkt = Vec::with_capacity(max_packet);
    let mut samples = Vec::with_capacity(max_frame_samples);

    pkt.resize(max_packet, 0u8);
    samples.resize(max_frame_samples, 0i16);

    let mut buf = [0u8; 4];
    while in_f.read_exact(&mut buf).is_ok() {
        let len = get_i32b(&buf) as usize;
        if len > max_packet {
            panic!("Impossible packet size {}", len);
        }

        in_f.read_exact(&mut buf).expect("End of file");

        in_f.read_exact(&mut pkt[..len]).expect("End of file");

        let ret = unsafe {
            opus_decode(dec,
                        pkt.as_ptr(), pkt.len() as i32,
                        samples.as_mut_ptr(), samples.len() as i32, 0)
        };

        if ret > 0 {
            // Write the actual group of samples
            let out = unsafe { slice::from_raw_parts(samples.as_ptr() as *const u8, ret as usize * 2) };
            out_f.write_all(out).unwrap();
        } else {
            panic!("Cannot decode");
        }
    }
}
