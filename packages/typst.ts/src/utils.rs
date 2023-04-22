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

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
macro_rules! console_log {
    ($($arg:tt)*) => {
        println!(
            $($arg)*
        );
    }
}

pub(crate) use console_log; // <-- the trick
