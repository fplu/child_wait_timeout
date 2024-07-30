use std::io;
use std::process::Child;

use libc::{pid_t, ETIMEDOUT};

use crate::error::{_generate_default_error, _generate_timeout_error};

extern "C" {
    fn wait_timeout_untraced(pid: pid_t, timeout_ms: u32) -> pid_t;
}

pub(crate) fn _wait_timeout_untraced_ms(child: &mut Child, timeout_ms: u32) -> io::Result<()> {
    let pid = child.id() as pid_t;

    let result = unsafe { wait_timeout_untraced(pid, timeout_ms) };
    if result != -1 {
        Ok(())
    } else if std::io::Error::last_os_error().raw_os_error() == Some(ETIMEDOUT) {
        _generate_timeout_error()
    } else {
        _generate_default_error()
    }
}
