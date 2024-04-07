use std::path::PathBuf;

use anyhow::Result;

const BOUNDARIES: [&str; 4] = ["MODULE.bazel", "REPO.bazel", "WORKSPACE.bazel", "WORKSPACE"];

pub fn get_root(cwd: &PathBuf) -> Result<PathBuf> {
    let mut root = cwd.clone();
    loop {
        for boundary in BOUNDARIES {
            if cwd.join(boundary).exists() {
                return Ok(root)
            }
        }
        let parent = root.parent();
        if parent.is_none() {
            break
        }
        root = parent.unwrap().to_path_buf()
    }
    anyhow::bail!("Not inside a workspace")
}