extern crate opus_sys as ffi;

#[cfg(feature="codec-trait")]
extern crate av_data as data;

#[cfg(feature="codec-trait")]
extern crate av_codec as codec;

#[cfg(feature="codec-trait")]
extern crate av_bitstream as bitstream;

pub mod common;
pub mod encoder;
pub mod decoder;
