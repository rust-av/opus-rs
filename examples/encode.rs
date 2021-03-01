use libopus::encoder::*;

use structopt::StructOpt;

use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

use av_bitstream::bytewrite::put_i32b;

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
    sampling_rate: usize,
    /// Channels, either 1 or 2
    #[structopt(default_value = "1")]
    channels: usize,
    /// Bitrate
    #[structopt(default_value = "16000")]
    bits_per_second: u32,
    /// Number of seconds to encode
    #[structopt(default_value = "10")]
    seconds: usize,
}

trait Encode {
    fn get_encoder(&self) -> Option<Encoder>;
}

impl Encode for EncodingOpts {
    fn get_encoder(&self) -> Option<Encoder> {
        if self.channels > 2 {
            unimplemented!("Multichannel support")
        }

        let coupled_streams = if self.channels > 1 { 1 } else { 0 };

        Encoder::create(self.sampling_rate, self.channels, 1, coupled_streams, &[0u8, 1u8], Application::Audio).ok().map(|mut enc| {
            enc.set_option(OPUS_SET_BITRATE_REQUEST, self.bits_per_second).unwrap();
            enc.set_option(OPUS_SET_BANDWIDTH_REQUEST, OPUS_BANDWIDTH_WIDEBAND).unwrap();
            enc.set_option(OPUS_SET_COMPLEXITY_REQUEST, 10).unwrap();
            enc.set_option(OPUS_SET_VBR_REQUEST, 0).unwrap();
            enc.set_option(OPUS_SET_VBR_CONSTRAINT_REQUEST, 0).unwrap();
            enc.set_option(OPUS_SET_PACKET_LOSS_PERC_REQUEST, 0).unwrap();
            enc
        })
    }
}

use std::slice;

fn main() {
    let enc_opt = EncodingOpts::from_args();

    let mut enc = enc_opt.get_encoder().unwrap();

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

        let samples: &[i16] = unsafe {
            slice::from_raw_parts(buf.as_ptr() as *const i16, frame_size)
        };

        processed_bytes += frame_size * 2;


        if let Ok(ret) = enc.encode(samples, &mut out_buf) {
            let mut b = [0u8; 4];
            // Write the packet size
            put_i32b(&mut b, ret as i32);
            out_f.write_all(&b).unwrap();

            // Write the encoder ec final state
            let val = enc.get_option(OPUS_GET_FINAL_RANGE_REQUEST).unwrap();
            put_i32b(&mut b, val);
            out_f.write_all(&b).unwrap();

            // Write the actual packet
            out_f.write_all(&out_buf[..ret]).unwrap();
        } else {
            panic!("Cannot encode");
        }
    }
}
