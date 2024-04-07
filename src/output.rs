use std::path::PathBuf;

#[cfg(target_os = "macos")]
const TMP: &str = "/var/tmp";

pub fn get_default_user_output_root() -> PathBuf {
    let user = whoami::username();
    let mut tmp = PathBuf::from(TMP);
    tmp.push(format!("_bazel_{}", user));
    tmp
}

pub fn get_default_output_base(output_root: &PathBuf, root: &PathBuf) -> PathBuf {
    let digest = md5::compute(root.to_string_lossy().as_bytes());
    output_root.join(format!("{:x}", digest))
}
