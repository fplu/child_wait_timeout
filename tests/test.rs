extern crate utilities;

#[cfg(test)]
mod tests {
    use child_wait_timeout::ChildWT;
    use std::{io, time::Duration};
    use utilities;

    #[test]
    fn test_wait_timeout_success() {
        // Spawn a short-lived process
        let mut child = utilities::sleep_child("1");

        // Wait for the process to exit with a timeout
        let result = child.wait_timeout(Duration::from_secs(5));

        // Verify that the process exited successfully
        assert!(result.is_ok());
    }

    #[test]
    fn test_wait_timeout_very_big_success() {
        // Spawn a short-lived process
        let mut child = utilities::sleep_child("1");

        // Wait for the process to exit with a timeout
        let result = child.wait_timeout(Duration::from_secs(4_294_967_295u64));

        // Verify that the process exited successfully
        assert!(result.is_ok());
    }

    #[test]
    fn test_wait_timeout_exceeded() {
        // Spawn a long-running process
        let mut child = utilities::sleep_child("3");

        // Wait for the process to exit with a short timeout
        let result = child.wait_timeout(Duration::from_secs(1));

        // Verify that the timeout was exceeded
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().kind(),
            io::ErrorKind::TimedOut
        ));
    }

    #[test]
    fn test_wait_timeout_multiple_success() {
        // Spawn a short-lived process
        let mut child = utilities::sleep_child("1");

        // Wait for the process to exit with a timeout
        let result = child.wait_timeout(Duration::from_secs(5));

        // Verify that the process exited successfully
        assert!(result.is_ok());

        // Wait for the process to exit with a timeout
        let result2 = child.wait_timeout(Duration::from_secs(5));

        assert_eq!(result.unwrap(), result2.unwrap());
    }
}
