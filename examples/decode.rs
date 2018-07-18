extern crate libopus;
#[macro_use]
extern crate structopt;
extern crate av_bitstream as bitstream;

use libopus::decoder::*;

use structopt::StructOpt;

use std::mem;

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
    sampling_rate: usize,
    /// Channels, either 1 or 2
    #[structopt(default_value = "1")]
    channels: usize,
    /// Number of seconds to decode
    #[structopt(default_value = "10")]
    seconds: i32,
}

trait Decode {
    fn get_decoder(&self) -> Option<Decoder>;
}

impl Decode for DecodingOpts {
    fn get_decoder(&self) -> Option<Decoder> {
        if self.channels > 2 {
            unimplemented!("Multichannel support missing");
        }

        let coupled_streams = if self.channels > 1 { 1 } else { 0 };
        Decoder::create(self.sampling_rate, self.channels, 1, coupled_streams, &[0u8, 1u8]).ok()
    }
}

use std::slice;

fn main() {
    let dec_opt = DecodingOpts::from_args();

    let mut dec = dec_opt.get_decoder().unwrap();

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

        if let Ok(ret) = dec.decode(&pkt[..len], &mut samples[..], false) {
            // Write the actual group of samples
            let out = unsafe { slice::from_raw_parts(mem::transmute(samples.as_ptr()), ret as usize * 2) };
            out_f.write_all(out).unwrap();
        } else {
            panic!("Cannot decode");
        }
    }
}
