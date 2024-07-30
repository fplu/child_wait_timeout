use std::{
    io,
    process::{Child, ExitStatus},
    time::Duration,
};

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

#[cfg(all(unix, feature = "pidfd"))]
#[path = "unix_pidfd.rs"]
mod imp;

#[cfg(all(
    unix,
    any(
        all(feature = "thread", not(feature = "pidfd")),
        all(
            not(feature = "signal"),
            not(feature = "thread"),
            not(feature = "pidfd")
        )
    )
))]
#[path = "unix_thread.rs"]
mod imp;

#[cfg(all(
    unix,
    feature = "signal",
    not(feature = "thread"),
    not(feature = "pidfd")
))]
#[path = "unix_signal.rs"]
mod imp;

pub trait ChildWT {
    /// Waits for the child process to exit or until the timeout expires.
    ///
    /// # Parameters
    /// - `timeout`: The maximum time to wait for the child process to exit. This function handles durations exceeding `u32::MAX` milliseconds.
    ///
    /// # Returns
    /// - `Result<ExitStatus>`:
    ///   - `Ok(ExitStatus)` if the child process exits successfully.
    ///   - `Err` with `ErrorKind::TimedOut` if the timeout expires before the child process exits.
    ///
    /// # Platform-Specific Behavior
    ///
    /// ## Windows
    ///
    /// This function is implemented using `WaitForSingleObject`.
    ///
    /// ## Unix
    ///
    /// This function is implemented using `thread` if no features are specified. Otherwise, it uses either `pidfd`, `signal`, or `thread`
    /// depending on the feature flag specified. If multiple features are explicitly selected, the priority order is:
    /// `pidfd`, then `thread`, and finally `signal`.
    /// For more details on feature unification, refer to the [Rust documentation](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification).
    ///
    /// ### Priority Order Rationale
    ///
    /// - **`pidfd`**: Prioritized because if a crate depends on Linux 5.3 or later, it is safe to always use `pidfd`. It provides lower overhead and efficient handling of child process termination.
    /// - **`thread`**: Chosen next due to its moderate overhead and reliability.
    /// - **`signal`**: Selected last due to potential issues with race conditions and higher overhead. It allows `signal` to be forcefully disabled if a crate uses incompatible signal handling.
    ///
    ///
    /// # Feature Implementation Details
    ///
    /// ## `pidfd`
    ///
    /// The `pidfd` feature uses `pidfd_open` to wait for the child process.
    /// This is a relatively new and efficient method available on newer Linux kernels (5.3 and later).
    ///
    /// ### Benchmark
    /// See [Benchmark Results](#benchmark-results).
    ///
    /// ## `thread`
    ///
    /// If no features are specified, the default implementation is `thread`.
    /// It uses a separate thread that waits for the child process to exit and cancels the wait if the timeout expires.
    /// It is POSIX compliant.
    ///
    /// ### Benchmark
    /// See [Benchmark Results](#benchmark-results).
    ///
    /// ## `signal`
    ///
    /// The `signal` feature uses `SIGCHLD` to detect child process termination.
    /// This method can introduce complexity and potential race conditions in signal handling.
    /// It is POSIX compliant.
    ///
    /// ### Benchmark
    /// See [Benchmark Results](#benchmark-results).
    ///
    ///
    /// # Errors
    ///
    /// This function may fail with `ErrorKind::TimedOut` if the specified duration expires before the child process exits.
    ///
    /// # Example
    /// ```rust
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use std::time::Duration;
    /// use std::process::{Command, ExitStatus};
    ///
    /// use child_wait_timeout::ChildWT;
    ///
    /// let mut child = if cfg!(target_os = "windows") {
    ///     Command::new("timeout").args(["/t", "2"]).spawn()?
    /// } else {
    ///     Command::new("sleep").arg("2").spawn()?
    /// };
    /// let status = child.wait_timeout(Duration::from_secs(2));
    ///
    /// match status {
    ///     Ok(exit_status) => println!("Process exited with status: {:?}", exit_status),
    ///     Err(e) if e.kind() == std::io::ErrorKind::TimedOut => println!("Process timed out"),
    ///     Err(e) => println!("Failed to wait on process: {:?}", e),
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Benchmark Results
    ///
    /// Mean execution time on 100 measurement when executing a "sleep 0" and waiting in millisecond.
    ///
    /// | Method                      | Time (ms) | Percent  |
    /// |-----------------------------|-----------|----------|
    /// | wait                        | 4.4810    | 100.00%  |
    /// | wait_timeout with pidfd     | 4.4646    | 99.63%   |
    /// | wait_timeout with thread    | 4.4688    | 99.73%   |
    /// | wait_timeout with signal    | 4.4861    | 100.11%  |
    ///
    /// Mean execution time on 100 measurement time when executing a "sleep 1" and waiting in millisecond
    ///
    /// | Method                      | Time (ms) | Percent  |
    /// |-----------------------------|-----------|----------|
    /// | wait                        | 1.0236    | 100.00%  |
    /// | wait_timeout with pidfd     | 1.0228    | 99.92%   |
    /// | wait_timeout with thread    | 1.0237    | 100.01%  |
    /// | wait_timeout with signal    | 1.0233    | 99.97%   |
    ///  
    /// Mean execution time on 100 measurement time when executing a "sleep 1000" and timeouting after 1 second in millisecond
    ///
    /// | Method                      | Time (ms) | Percent  |
    /// |-----------------------------|-----------|----------|
    /// | sleep                       | 1.0004    | 100.00%  |
    /// | wait_timeout with pidfd     | 1.0013    | 100.09%  |
    /// | wait_timeout with thread    | 1.0005    | 100.01%  |
    /// | wait_timeout with signal    | 1.0004    | 100.00%  |
    ///
    /// ## Interpretation
    ///
    /// The data shows that all methods are equivalent in terms of execution speed.
    /// The differences in execution times are not significant, indicating that any of the methods can be used interchangeably without impacting performance.
    /// The performance overheads seem negligible compared to process creation.
    ///
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<ExitStatus>;
}

impl ChildWT for Child {
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<ExitStatus> {
        if let Ok(Some(res)) = self.try_wait() {
            return Ok(res);
        }

        const U32_MAX: u128 = u32::MAX as u128;
        let mut timeout_ms = timeout.as_millis();

        while timeout_ms > U32_MAX {
            match imp::_wait_timeout_untraced_ms(self, u32::MAX) {
                // the child.wait will end instantly
                Ok(()) => return self.wait(),
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    // continue looping
                }
                Err(e) => return Err(e),
            };
            timeout_ms -= U32_MAX;
        }
        imp::_wait_timeout_untraced_ms(self, timeout_ms as u32)?;
        // the child.wait will end instantly
        self.try_wait().and_then(|v| Ok(v.expect("aa")))
    }
}
