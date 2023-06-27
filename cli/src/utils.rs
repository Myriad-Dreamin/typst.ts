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

/// Exit status code used for successful compilation and help output.
pub const EXIT_SUCCESS: i32 = 0;

/// Exit status code used for compilation failures and invalid flags.
pub const EXIT_FAILURE: i32 = 1;

pub fn logical_exit(is_success: bool) -> ! {
    std::process::exit(if is_success {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    })
}
