use anyhow::{anyhow, Result};

use crate::{err, ok};

pub fn get_docker_platform() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform = match (os, arch) {
        ("linux", "x86_64") => "linux/amd64",
        ("linux", "aarch64") => "linux/arm64",
        ("macos", "x86_64") => "linux/amd64",
        ("macos", "aarch64") => "linux/arm64",
        ("windows", "x86_64") => "windows/amd64",
        ("windows", "aarch64") => "windows/arm64",
        _ => err!(anyhow!("Unsupported platform")),
    };

    ok!(platform.to_string())
}
