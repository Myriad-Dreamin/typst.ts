#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[allow(unused_macros)]
macro_rules! console_log {
    ($($arg:tt)*) => {
        web_sys::console::info_1(&format!(
            $($arg)*
        ).into());
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[allow(unused_macros)]
macro_rules! console_log {
    ($($arg:tt)*) => {
        println!(
            $($arg)*
        );
    }
}

#[allow(unused_imports)]
pub(crate) use console_log;

pub(crate) struct EmptyFuture;
impl core::future::Future for EmptyFuture {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<()> {
        core::task::Poll::Ready(())
    }
}
