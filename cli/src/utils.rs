pub fn async_run<F: std::future::Future<Output = ()>>(f: F) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f);
}
