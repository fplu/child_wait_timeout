# Child Process Wait Timeout Crate

## Overview

The Child Process Wait Timeout crate provides a straightforward and efficient way to wait for a child process to exit with an optional timeout. 
It supports multiple implementation methods depending on the platform and available features, ensuring optimal performance and reliability across different environments.

## Features
- **Timeout Handling:** Allows specifying a maximum wait time for a child process to exit.
- **Cross-Platform Support:** Seamlessly works on both Windows and Unix systems.
- **Multiple Implementation Methods:**
  - **Windows:** Utilizes `WaitForSingleObject`.
  - **Unix:** Utilizes `thread` by default. User can choose between `pidfd`, `thread`, or `signal` based on feature flags, with a priority order of `pidfd`, then `thread`, and finally `signal`.
- **Performance:** Minimal overhead, with benchmark results indicating negligible differences between methods.
- **Error Handling:** Provides clear error messages, including timeout errors.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
child_wait_timeout = "0.1.0"
```

## Usage

Here's a basic example of how to use the crate:

```rust
use std::time::Duration;
use std::process::Command;
use child_wait_timeout::ChildWT;

fn main() {
    let mut child = if cfg!(target_os = "windows") {
        Command::new("timeout").args(["/t", "2"]).spawn()?
    } else {
        Command::new("sleep").arg("2").spawn()?
    };
    
    let status = child.wait_timeout(Duration::from_secs(2));

    match status {
        Ok(exit_status) => println!("Process exited with status: {:?}", exit_status),
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => println!("Process timed out"),
        Err(e) => println!("Failed to wait on process: {:?}", e),
    }
}
```

## Platform-Specific Behavior

### Windows
Implemented using `WaitForSingleObject`.

### Unix
Implemented using either `pidfd`, `thread`, or `signal` depending on the feature flag specified. The priority order is:
1. **`pidfd`**: Linux 5.3 and later compliant and straightforward implementation.
2. **`thread`**: Posix compliant.
3. **`signal`**: Posix compliant but have side effects.

## Benchmark Results

Mean execution time on 100 measurements (in milliseconds):

### For "sleep 0"
| Method                      | Time (ms) | Percent  |
|-----------------------------|-----------|----------|
| wait                        | 4.4810    | 100.00%  |
| wait_timeout with pidfd     | 4.4646    | 99.63%   |
| wait_timeout with thread    | 4.4688    | 99.73%   |
| wait_timeout with signal    | 4.4861    | 100.11%  |

### For "sleep 1"
| Method                      | Time (ms) | Percent  |
|-----------------------------|-----------|----------|
| wait                        | 1.0236    | 100.00%  |
| wait_timeout with pidfd     | 1.0228    | 99.92%   |
| wait_timeout with thread    | 1.0237    | 100.01%  |
| wait_timeout with signal    | 1.0233    | 99.97%   |

### For "sleep 1000" with 1-second timeout
| Method                      | Time (ms) | Percent  |
|-----------------------------|-----------|----------|
| sleep                       | 1.0004    | 100.00%  |
| wait_timeout with pidfd     | 1.0013    | 100.09%  |
| wait_timeout with thread    | 1.0005    | 100.01%  |
| wait_timeout with signal    | 1.0004    | 100.00%  |

## Interpretation

The data shows that all methods are equivalent in terms of execution speed. 
The differences in execution times are not significant, indicating that any of the methods can be used interchangeably without impacting performance. 
The performance overheads seem negligible compared to process creation.

## License

This project is licensed under the MIT License. 
See the [LICENSE](LICENSE) file for details.

---

Feel free to open issues or pull requests if you find any bugs or have feature requests. Contributions are welcome!