use std::process::{Child, Command, Stdio};

pub fn sleep_command(time: &str) -> Command {
    #[cfg(windows)]
    {
        let mut command = Command::new("timeout.exe");
        command.args(["/t", time]).stdout(Stdio::null());
        command
    }

    #[cfg(unix)]
    {
        let mut command = Command::new("sleep");
        command.arg(time).stdout(Stdio::null());
        command
    }
}

pub fn sleep_child(time: &str) -> Child {
    sleep_command(time)
        .spawn()
        .expect("Failed to start sleep command")
}
