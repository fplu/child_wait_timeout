//! # Child Wait Timeout Crate
//!
//! The Child Wait Timeout crate provides a simple and efficient way to wait for a child process to exit with a timeout. It supports different implementations depending on the platform and available features, ensuring performance, scalability and reliability.
//!
//! ## Features
//! - **Timeout Handling:** Wait for a child process to exit within a specified timeout.
//! - **Cross-Platform Support:** Works seamlessly on both Windows and Unix systems.
//! - **Multiple Implementation Methods:**
//!   - **Windows:** Uses `WaitForSingleObject`.
//!   - **Unix:** Uses a method based on `thread_cancel` by default, see the wait_timeout documentation for more detail.
//! - **Performance Benchmarks:** Minimal performance overhead, with detailed benchmark results showing negligible differences between methods.
//! - **Error Handling:** Provides clear error messages, including timeout errors.
//!
//! ## Example
//!
//! ```rust
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::time::Duration;
//! use std::process::Command;
//! use child_wait_timeout::ChildWT;
//!
//! let mut child = if cfg!(target_os = "windows") {
//!     Command::new("timeout").args(["/t", "2"]).spawn()?
//! } else {
//!     Command::new("sleep").arg("2").spawn()?
//! };
//! let status = child.wait_timeout(Duration::from_secs(2));
//!
//! match status {
//!     Ok(exit_status) => println!("Process exited with status: {:?}", exit_status),
//!     Err(e) if e.kind() == std::io::ErrorKind::TimedOut => println!("Process timed out"),
//!     Err(e) => println!("Failed to wait on process: {:?}", e),
//! }
//! #     Ok(())
//! # }
//! ```
//!
//! With this crate, managing child process termination with timeouts becomes straightforward, efficient and scalable, making it an essential tool when dealing with process management.
//!
mod error;
mod platform;
pub use platform::*;
