#[macro_export]
macro_rules! serde_exporter {
    ($name:ident) => {
        pub struct $name {
            path: Option<std::path::PathBuf>,
        }

        impl $name {
            pub fn new_path(path: std::path::PathBuf) -> Self {
                Self { path: Some(path) }
            }
        }
    };
}
