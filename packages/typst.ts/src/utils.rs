#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
macro_rules! console_log {
    ($($arg:tt)*) => {
        let v: JsValue = format!(
            $($arg)*
        )
        .into();
        console::info_1(&v);
    }
}

#[allow(unused_macros)]
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
macro_rules! console_log {
    ($($arg:tt)*) => {
        println!(
            $($arg)*
        );
    }
}

#[allow(unused_imports)]
pub(crate) use console_log; // <-- the trick
