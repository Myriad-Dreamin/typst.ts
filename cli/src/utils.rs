use tokio::runtime::Builder;

pub fn async_continue<F: std::future::Future<Output = ()>>(f: F) -> ! {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f);

    #[allow(unreachable_code)]
    {
        unreachable!("The async command must exit the process.");
    }
}
