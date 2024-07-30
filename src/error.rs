use std::io;

pub(crate) fn _generate_default_error() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "an unspecified error occurred",
    ))
}

pub(crate) fn _generate_timeout_error() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "operation timed out",
    ))
}
