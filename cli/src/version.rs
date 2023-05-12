use std::process::exit;

use typst_ts_core::build_info::VERSION;

pub fn intercept_version(print_version: bool) {
    if print_version {
        // todo: global app name
        println!("typst-ts-cli {}", VERSION);
        exit(0);
    }
}
