use anyhow::Result;
use std::{fs, path::PathBuf};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const PLATFORM: &str = "darwin-arm64";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const PLATFORM: &str = "darwin-amd64";

pub struct BazelVersion {
    src: String,
    version: String,
}

impl BazelVersion {
    fn executable_name(&self) -> String {
        format!("bazel-{}-{}", self.version, PLATFORM)
    }
    fn dir(&self) -> PathBuf {
        dirs::cache_dir()
            .unwrap()
            .join("bazelisk/downloads")
            .join(&self.src)
            .join(self.executable_name())
    }

    pub fn exists(&self) -> bool {
        self.dir().exists()
    }

    pub fn get(&self) -> PathBuf {
        let dir = self.dir();
        let bin = dir.join("bin/bazel");
        if !dir.exists() {
            let uri = format!(
                "https://github.com/{}/bazel/releases/download/{}/{}",
                &self.src,
                &self.version,
                &self.executable_name()
            );
            fs::create_dir_all(bin.parent().unwrap()).expect("failed to create bazel directory");
            let mut file = fs::File::create(&bin).expect("failed to create bazel");
            http_req::request::get(uri, &mut file).expect("failed to download bazel");
        }
        println!("{:?}", bin);
        bin
    }

    pub fn from_root(root: &PathBuf) -> Result<BazelVersion> {
        let bazelversion = root.join(".bazelversion");
        let mut version = String::from("7.1.1");
        if bazelversion.exists() {
            version = fs::read_to_string(bazelversion)?;
        }
        Ok(BazelVersion {
            src: "bazelbuild".into(),
            version,
        })
    }
}
