use std::{path::Path, sync::Arc};

use parking_lot::Mutex;
use typst::{
    diag::{PackageError, PackageResult},
    file::PackageSpec,
};

use super::{DummyNotifier, Notifier, Registry};

pub struct SystemRegistry {
    notifier: Arc<Mutex<dyn Notifier + Send>>,
}

impl Default for SystemRegistry {
    fn default() -> Self {
        Self {
            notifier: Arc::new(Mutex::<DummyNotifier>::default()),
        }
    }
}

impl SystemRegistry {
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
    pub fn prepare_package(&self, spec: &PackageSpec) -> PackageResult<Arc<Path>> {
        let subdir = format!(
            "typst/packages/{}/{}-{}",
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
            if !dir.exists() {
                self.download_package(spec, &dir)?;
            }

            if dir.exists() {
                return Ok(dir.into());
            }
        }

        Err(PackageError::NotFound(spec.clone()))
    }

    /// Download a package over the network.
    fn download_package(&self, spec: &PackageSpec, package_dir: &Path) -> PackageResult<()> {
        let url = format!(
            "https://packages.typst.org/preview/{}-{}.tar.gz",
            spec.name, spec.version
        );

        self.notifier.lock().downloading(spec);
        let reader = match agent().get(&url).call() {
            Ok(response) => response.into_reader(),
            Err(ureq::Error::Status(404, _)) => return Err(PackageError::NotFound(spec.clone())),
            Err(_) => return Err(PackageError::NetworkFailed),
        };

        let decompressed = flate2::read::GzDecoder::new(reader);
        tar::Archive::new(decompressed)
            .unpack(package_dir)
            .map_err(|_| {
                std::fs::remove_dir_all(package_dir).ok();
                PackageError::MalformedArchive
            })
    }
}

impl Registry for SystemRegistry {
    fn resolve(&self, spec: &PackageSpec) -> PackageResult<std::sync::Arc<Path>> {
        self.prepare_package(spec)
    }
}

#[cfg(not(target_arch = "riscv64"))]
fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().build()
}

#[cfg(target_arch = "riscv64")]
fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .tls_connector(Arc::new(native_tls::TlsConnector::new().unwrap()))
        .build()
}
