//!https://github.com/akiradeveloper/git-download/blob/8398f9091c1bb32642b7dfac3138ee593c3fee93/git-download/src/lib.rs

use cmd_lib::run_cmd;
use sha2::Digest;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

/// Create a downloader whose target is the repository.
/// You can use any form of url which is allowed in `git remote add`.
/// e.g. git@github.com:akiradeveloper/git-download
pub fn repo(path: impl Into<String>) -> Downloader {
    Downloader::new(path)
}

#[derive(Debug)]
struct CopyRequest {
    from: PathBuf,
    to: PathBuf,
    hash: Option<String>,
}
#[derive(Debug)]
pub struct Downloader {
    repo_path: String,
    branch_name: String,
    out_dir: PathBuf,
    copy_requests: Vec<CopyRequest>,
}
impl Downloader {
    fn new(repo: impl Into<String>) -> Self {
        let cur_dir = std::env::current_dir().unwrap();
        Self {
            repo_path: repo.into(),
            branch_name: "master".to_owned(),
            out_dir: cur_dir,
            copy_requests: vec![],
        }
    }
    /// Change the output directory. The default is the current dir.
    pub fn out_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.out_dir = path.as_ref().to_owned();
        self
    }
    /// Change the branch name. The default is "master".
    pub fn branch_name(mut self, name: impl Into<String>) -> Self {
        self.branch_name = name.into();
        self
    }
    /// Add a file to copy from remote to local.
    /// The path `src` is a relative from the repository root and the
    /// path `dst` is a relative from `out_dir`.
    pub fn add_file_verified(
        mut self,
        src: impl AsRef<Path>,
        dst: impl AsRef<Path>,
        hash: Option<String>,
    ) -> Self {
        let from = src.as_ref().to_owned();
        let to = dst.as_ref().to_owned();
        let req = CopyRequest { from, to, hash };
        self.copy_requests.push(req);
        self
    }

    pub fn add_file(self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Self {
        self.add_file_verified(src, dst, None)
    }
    /// Execute downloading.
    pub fn exec(self) -> anyhow::Result<()> {
        let old_pwd = std::env::current_dir()?;

        let dir = tempfile::tempdir()?;
        let dir_path = dir.path();
        std::env::set_current_dir(dir_path)?;

        let repo = &self.repo_path;
        run_cmd! {
            git init .;
            git config core.sparsecheckout true;
            git remote add origin $repo;
        }?;

        let checkout_file = std::path::Path::new(".git/info/sparse-checkout");
        // create parent directories
        let parent = checkout_file.parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        for req in &self.copy_requests {
            let from_path = &req.from;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(checkout_file)?;

            file.write_all(from_path.display().to_string().as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }

        let branch_name = &self.branch_name;
        run_cmd! {
            git pull origin $branch_name;
        }?;

        for req in &self.copy_requests {
            let from = &req.from;
            let to = &req.to;
            let to = self.out_dir.join(to);
            let to_dir = to.parent().unwrap();
            if !to_dir.exists() {
                std::fs::create_dir_all(to_dir)?;
            }

            print!("check file exists({:?}): {}", from.exists(), from.display());
            std::fs::copy(from, to.clone())?;
            let content = std::fs::read(to.clone())?;
            let sha_hash = hex::encode(sha2::Sha256::digest(&content));
            println!(" .. {}", sha_hash);
            if let Some(hash) = &req.hash {
                if *hash != sha_hash {
                    anyhow::bail!(
                        "hash mismatch for {:?} (expected: {}, actual: {})",
                        to,
                        hash,
                        sha_hash
                    );
                }
            }
        }

        std::env::set_current_dir(old_pwd)?;
        Ok(())
    }
}
