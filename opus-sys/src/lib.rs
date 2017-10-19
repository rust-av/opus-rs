// TODO do w/out the unions?
#![feature(untagged_unions)]

pub mod opus;

#[cfg(test)]
mod tests {
    use super::opus::*;
    use std::ffi::CStr;
    #[test]
    fn version() {
        println!("{}", unsafe {
            CStr::from_ptr(opus_get_version_string()).to_string_lossy()
        });
    }
}
