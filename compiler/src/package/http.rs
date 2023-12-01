use std::{path::Path, sync::Arc};

use parking_lot::Mutex;
use typst::diag::eco_format;

use super::{DummyNotifier, Notifier, PackageError, PackageSpec, Registry};

pub struct HttpRegistry {
    notifier: Arc<Mutex<dyn Notifier + Send>>,
}

impl Default for HttpRegistry {
    fn default() -> Self {
        Self {
            notifier: Arc::new(Mutex::<DummyNotifier>::default()),
        }
    }
}

impl HttpRegistry {
    pub fn local_path(&self) -> Option<Box<Path>> {
        if let Some(data_dir) = dirs::data_dir() {
            if data_dir.exists() {
                return Some(data_dir.join("typst/packages").into());
            }
        }

        None
    }

    pub fn paths(&self) -> Vec<Box<Path>> {
        let mut res = vec![];
        if let Some(data_dir) = dirs::data_dir() {
            let dir: Box<Path> = data_dir.join("typst/packages").into();
            if dir.exists() {
                res.push(dir);
            }
        }

        if let Some(cache_dir) = dirs::cache_dir() {
            let dir: Box<Path> = cache_dir.join("typst/packages").into();
            if dir.exists() {
                res.push(dir);
            }
        }

        res
    }

    /// Make a package available in the on-disk cache.
    pub fn prepare_package(&self, spec: &PackageSpec) -> Result<Arc<Path>, PackageError> {
        let subdir = format!(
            "typst/packages/{}/{}/{}",
            spec.namespace, spec.name, spec.version
        );

        if let Some(data_dir) = dirs::data_dir() {
            let dir = data_dir.join(&subdir);
            if dir.exists() {
                return Ok(dir.into());
            }
        }

        if let Some(cache_dir) = dirs::cache_dir() {
            let dir = cache_dir.join(&subdir);

            // Download from network if it doesn't exist yet.
            if spec.namespace == "preview" && !dir.exists() {
                self.download_package(spec, &dir)?;
            }

            if dir.exists() {
                return Ok(dir.into());
            }
        }

        Err(PackageError::NotFound(spec.clone()))
    }

    /// Download a package over the network.
    fn download_package(&self, spec: &PackageSpec, package_dir: &Path) -> Result<(), PackageError> {
        let url = format!(
            "https://packages.typst.org/preview/{}-{}.tar.gz",
            spec.name, spec.version
        );

        self.notifier.lock().downloading(spec);
        let client = reqwest::blocking::Client::builder().build().unwrap();
        let reader = match client.get(url).send() {
            Ok(response) => response,
            Err(err) if matches!(err.status().map(|s| s.as_u16()), Some(404)) => {
                return Err(PackageError::NotFound(spec.clone()))
            }
            Err(err) => return Err(PackageError::NetworkFailed(Some(eco_format!("{err}")))),
        };

        let decompressed = flate2::read::GzDecoder::new(reader);
        tar::Archive::new(decompressed)
            .unpack(package_dir)
            .map_err(|err| {
                std::fs::remove_dir_all(package_dir).ok();
                PackageError::MalformedArchive(Some(eco_format!("{err}")))
            })
    }
}

impl Registry for HttpRegistry {
    fn resolve(&self, spec: &PackageSpec) -> Result<std::sync::Arc<Path>, PackageError> {
        self.prepare_package(spec)
    }
}
