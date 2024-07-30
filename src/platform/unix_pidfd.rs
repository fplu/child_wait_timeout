use std::io;
use std::mem;
use std::os::unix::io::RawFd;
use std::process::Child;
use std::ptr;

use libc::{c_uint, close, pid_t, select, suseconds_t, time_t, timeval};

use crate::error::{_generate_default_error, _generate_timeout_error};

fn pidfd_open(pid: pid_t, flags: c_uint) -> io::Result<RawFd> {
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, flags) };
    if fd < 0 {
        _generate_default_error()?;
    }
    Ok(fd as RawFd)
}

pub(crate) fn _wait_timeout_untraced_ms(child: &mut Child, timeout_ms: u32) -> io::Result<()> {
    let pid = child.id() as pid_t;

    let pidfd = pidfd_open(pid, 0)?;

    // Convert the timeout to a timespec structure
    let mut tv = timeval {
        tv_sec: (timeout_ms / 1000) as time_t,
        tv_usec: (timeout_ms % 1000) as suseconds_t * 1000,
    };

    let mut fd_set: libc::fd_set = unsafe { mem::zeroed() };
    unsafe { libc::FD_SET(pidfd, &mut fd_set) };

    let result = unsafe {
        select(
            pidfd + 1,
            &mut fd_set,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut tv,
        )
    };

    unsafe { close(pidfd) };

    if result == -1 {
        _generate_default_error()
    } else if result == 0 {
        _generate_timeout_error()
    } else {
        Ok(())
    }
}
