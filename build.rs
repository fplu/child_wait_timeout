fn main() {
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
    {
        println!("cargo:rerun-if-changed=src/platform/wait_timeout_thread_untraced.c");

        cc::Build::new()
            .file("src/platform/wait_timeout_thread_untraced.c")
            .warnings(true)
            .flag("-Werror")
            .compile("wait_timeout_thread_untraced");
    }
}
