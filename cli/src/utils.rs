use std::{
    io,
    path::{Path, PathBuf},
};

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

pub fn exit_with_error<E: std::error::Error>(err: E) -> ! {
    clap::Error::raw(
        clap::error::ErrorKind::ValueValidation,
        format!("typst.ts error: {err}"),
    )
    .exit()
}

pub trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self) -> T;
}

impl<T, E: std::error::Error> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        self.map_err(exit_with_error).unwrap()
    }
}

pub fn symlink_dir(src: &Path, dst: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        use reflexo_typst::path::PathClean;
        let src = src.clean();
        let dst = dst.clean();
        // set up a junction, which is like a symlink dir but without the permission
        // requirements todo: filesystem other than NTFS?
        std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(dst)
            .arg(src)
            .output()
            // .map(|e| println!("{:?}", e))
            .map(|_| ())
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink as symlink_unix;
        symlink_unix(src, dst)
    }

    #[cfg(not(any(windows, unix)))]
    {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "symlinks are not supported on this platform",
        ))
    }
}

pub fn remove_symlink_dir(path: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        use reflexo_typst::path::PathClean;
        let path = path.clean();
        // remove a junction
        std::process::Command::new("cmd")
            .arg("/C")
            .arg("rmdir")
            .arg(path)
            .output()
            .map(|_| ())
    }

    #[cfg(unix)]
    {
        std::fs::remove_file(path)
    }

    #[cfg(not(any(windows, unix)))]
    {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "symlinks are not supported on this platform",
        ))
    }
}

pub fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_exit()
}

pub fn make_absolute_from(path: &Path, relative_to: impl FnOnce() -> PathBuf) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        relative_to().join(path)
    }
}

pub fn make_absolute(path: &Path) -> PathBuf {
    make_absolute_from(path, current_dir)
}
