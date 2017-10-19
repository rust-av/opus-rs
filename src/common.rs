use ffi::opus::*;
use std::fmt;
use std::ffi::CStr;
use std::i32;

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum ErrorCode {
    BadArg = OPUS_BAD_ARG,
    BufferTooSmall = OPUS_BUFFER_TOO_SMALL,
    InternalError = OPUS_INTERNAL_ERROR,
    InvalidPacket = OPUS_INVALID_PACKET,
    Unimplemented = OPUS_UNIMPLEMENTED,
    InvalidState = OPUS_INVALID_STATE,
    AllocFail = OPUS_ALLOC_FAIL,
    Unknown = i32::MAX,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = *self;
        let s = unsafe { CStr::from_ptr(opus_strerror(v as i32)) };
        write!(f, "{}", s.to_string_lossy())
    }
}

impl From<i32> for ErrorCode {
    fn from(v: i32) -> Self {
        use self::ErrorCode::*;
        match v {
            OPUS_BAD_ARG => BadArg,
            OPUS_BUFFER_TOO_SMALL => BufferTooSmall,
            OPUS_INTERNAL_ERROR => InternalError,
            OPUS_INVALID_PACKET => InvalidPacket,
            OPUS_UNIMPLEMENTED => Unimplemented,
            OPUS_INVALID_STATE => InvalidState,
            OPUS_ALLOC_FAIL => AllocFail,
            _ => Unknown,
        }
    }
}

pub enum AudioBuffer {
    F32(Vec<f32>),
    I16(Vec<i16>),
}

impl From<Vec<i16>> for AudioBuffer {
    fn from(v: Vec<i16>) -> Self {
        AudioBuffer::I16(v)
    }
}

impl From<Vec<f32>> for AudioBuffer {
    fn from(v: Vec<f32>) -> Self {
        AudioBuffer::F32(v)
    }
}
