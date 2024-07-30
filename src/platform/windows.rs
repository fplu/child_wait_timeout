use std::io;
use std::os::windows::io::AsRawHandle;
use std::process::Child;

use winapi::shared::winerror::WAIT_TIMEOUT;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::WAIT_OBJECT_0;

use crate::error::{_generate_default_error, _generate_timeout_error};

pub(crate) fn _wait_timeout_untraced_ms(child: &mut Child, timeout_ms: u32) -> io::Result<()> {
    let handle = child.as_raw_handle();

    let winapi_handle: *mut winapi::ctypes::c_void = handle as *mut winapi::ctypes::c_void;

    let result = unsafe { WaitForSingleObject(winapi_handle, timeout_ms) };

    if result == WAIT_TIMEOUT {
        _generate_timeout_error()
    } else if result == WAIT_OBJECT_0 {
        Ok(())
    } else {
        _generate_default_error()
    }
}
