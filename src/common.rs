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

pub enum AudioBuffer<'a> {
    F32(&'a [f32]),
    I16(&'a [i16]),
}

impl<'a> From<&'a [i16]> for AudioBuffer<'a> {
    fn from(v: &'a [i16]) -> Self {
        AudioBuffer::I16(v)
    }
}

impl<'a> From<&'a [f32]> for AudioBuffer<'a> {
    fn from(v: &'a [f32]) -> Self {
        AudioBuffer::F32(v)
    }
}

pub enum AudioBufferMut<'a> {
    F32(&'a mut [f32]),
    I16(&'a mut [i16]),
}

impl<'a> From<&'a mut [f32]> for AudioBufferMut<'a> {
    fn from(v: &'a mut [f32]) -> Self {
        AudioBufferMut::F32(v)
    }
}

impl<'a> From<&'a mut [i16]> for AudioBufferMut<'a> {
    fn from(v: &'a mut [i16]) -> Self {
        AudioBufferMut::I16(v)
    }
}
