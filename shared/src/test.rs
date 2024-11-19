pub fn init_tracing() {
    let _ = tracing_subscriber::fmt().pretty().try_init();
}
