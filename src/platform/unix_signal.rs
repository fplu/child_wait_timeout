use std::io;
use std::mem;
use std::process::Child;
use std::ptr;

use libc::{c_long, pid_t, sigtimedwait, time_t, timespec};
use libc::{sigemptyset, siginfo_t, SIGCHLD};

use crate::error::{_generate_default_error, _generate_timeout_error};

pub(crate) fn _wait_timeout_untraced_ms(child: &mut Child, timeout_ms: u32) -> io::Result<()> {
    let pid = child.id() as pid_t;

    // Convert the timeout to a timeval structure
    let ts = timespec {
        tv_sec: (timeout_ms / 1000) as time_t,
        tv_nsec: (timeout_ms % 1000) as c_long * 1000000,
    };

    // Set up the signal set to wait for SIGCHLD
    let mut sigset: libc::sigset_t = unsafe { mem::zeroed() };
    unsafe {
        sigemptyset(&mut sigset);
        libc::sigaddset(&mut sigset, SIGCHLD);
    }

    // Block SIGCHLD so it can be caught by sigtimedwait
    unsafe {
        libc::sigprocmask(libc::SIG_BLOCK, &sigset, ptr::null_mut());
    }

    // Wait for SIGCHLD with a timeout
    let mut siginfo: siginfo_t = unsafe { std::mem::zeroed() };
    let result = unsafe { sigtimedwait(&sigset, &mut siginfo, &ts) };

    if result == -1 {
        let errno = unsafe { *libc::__errno_location() };
        if errno == libc::EAGAIN {
            _generate_timeout_error()
        } else {
            _generate_default_error()
        }
    } else if unsafe { siginfo.si_pid() } == pid {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "another child died"))
    }
}
