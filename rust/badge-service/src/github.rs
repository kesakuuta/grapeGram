use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

use crate::error::{internal, ApiError};

pub fn publish_if_configured(data_dir: &Path, files: &[&str]) {
    let Ok(repo) = env::var("GRP_GITHUB_REPO") else {
        return;
    };
    if repo.trim().is_empty() {
        return;
    }
    for name in files {
        if let Err(error) = publish_one(&repo, data_dir, name) {
            eprintln!("github publish failed for {name}: {error}");
        }
    }
}

fn publish_one(repo: &str, data_dir: &Path, name: &str) -> Result<(), ApiError> {
    let bytes = fs::read(data_dir.join(name))?;
    let content = STANDARD.encode(bytes);
    let endpoint = format!("repos/{}/contents/data/{}", repo.trim(), name);

    let sha = Command::new("gh")
        .args(["api", endpoint.as_str(), "--jq", ".sha"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|sha| !sha.is_empty() && sha.as_str() != "null");

    let mut command = Command::new("gh");
    command
        .arg("api")
        .arg(&endpoint)
        .arg("-X")
        .arg("PUT")
        .arg("-f")
        .arg(format!("message=badge service update: {name}"))
        .arg("-f")
        .arg(format!("content={content}"))
        .arg("-f")
        .arg("branch=main");
    if let Some(sha) = sha {
        command.arg("-f").arg(format!("sha={sha}"));
    }
    if !command.status()?.success() {
        return Err(internal(format!("gh api rejected {name}")));
    }
    Ok(())
}
