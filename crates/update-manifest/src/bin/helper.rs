// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `cymbra-update-helper` — the Linux privileged update helper (change:
//! add-desktop-auto-update, design D5).
//!
//! ```text
//! pkexec /usr/lib/cymbra-music/cymbra-update-helper <envelope.json> <package.deb>
//! cymbra-update-helper --version
//! ```
//!
//! Runs as root behind the system password dialog. Both arguments were written
//! by a process running as the user, so both are hostile input:
//!
//! 1. read both into memory under size caps — every check below runs on these
//!    bytes, never on the paths again;
//! 2. decide with [`check_package`]: the helper's OWN compiled-in keys, product,
//!    target, size, SHA-256, strictly newer than the installed package;
//! 3. write the verified bytes into a fresh directory only root can write, and
//!    install THAT copy with `apt-get` — swapping the user's file after the
//!    password prompt changes nothing;
//! 4. remove the copy.
//!
//! Exit codes: 0 installed · 2 refused (nothing installed) · 3 install failed ·
//! 64 usage. The app maps anything but 0 to a localized "could not install".

use cymbra_update_manifest::helper::{HelperRefusal, check_package};
use cymbra_update_manifest::release_keys;

/// The Debian package this helper upgrades — and nothing else.
const PACKAGE_NAME: &str = "cymbra-music";
/// An envelope is a few hundred bytes; anything near this is not one.
const MAX_ENVELOPE_BYTES: u64 = 64 * 1024;
/// Far above any real package; the manifest's exact size is checked after.
const MAX_PACKAGE_BYTES: u64 = 1024 * 1024 * 1024;
/// Root-owned staging area for the verified copy.
const STAGING_ROOT: &str = "/var/cache/cymbra-music";

const EXIT_REFUSED: i32 = 2;
const EXIT_INSTALL_FAILED: i32 = 3;
const EXIT_USAGE: i32 = 64;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.as_slice() {
        [flag] if flag == "--version" => {
            println!("cymbra-update-helper {}", env!("CARGO_PKG_VERSION"));
            0
        }
        [envelope, package] => run(envelope, package),
        _ => {
            eprintln!("usage: cymbra-update-helper <envelope.json> <package.deb>");
            EXIT_USAGE
        }
    };
    std::process::exit(code);
}

#[cfg(unix)]
fn run(envelope_path: &str, package_path: &str) -> i32 {
    let envelope = match read_capped(envelope_path, MAX_ENVELOPE_BYTES) {
        Ok(b) => b,
        Err(e) => return refuse(&format!("envelope: {e}")),
    };
    let package = match read_capped(package_path, MAX_PACKAGE_BYTES) {
        Ok(b) => b,
        Err(e) => return refuse(&format!("package: {e}")),
    };
    let installed = match installed_version() {
        Some(v) => v,
        None => return refuse(&format!("{PACKAGE_NAME} is not installed")),
    };
    let trusted = match release_keys() {
        Ok(k) => k,
        Err(e) => return refuse(&format!("compiled-in keys: {e}")),
    };
    let manifest = match check_package(&envelope, &package, &trusted, &installed) {
        Ok(m) => m,
        Err(refusal) => return refuse(&describe(&refusal)),
    };

    let staged = match stage(&package) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("cymbra-update-helper: staging failed: {e}");
            return EXIT_INSTALL_FAILED;
        }
    };
    let deb = staged.join(format!("{PACKAGE_NAME}.deb"));
    let status = std::process::Command::new("apt-get")
        .args(["install", "-y", "--no-install-recommends"])
        .arg(&deb)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .status();
    let _ = std::fs::remove_dir_all(&staged);
    match status {
        Ok(s) if s.success() => {
            println!("cymbra-update-helper: installed {}", manifest.version);
            0
        }
        Ok(s) => {
            eprintln!("cymbra-update-helper: apt-get exited with {s}");
            EXIT_INSTALL_FAILED
        }
        Err(e) => {
            eprintln!("cymbra-update-helper: could not run apt-get: {e}");
            EXIT_INSTALL_FAILED
        }
    }
}

#[cfg(not(unix))]
fn run(_envelope_path: &str, _package_path: &str) -> i32 {
    eprintln!("cymbra-update-helper: Linux only");
    EXIT_USAGE
}

fn refuse(why: &str) -> i32 {
    eprintln!("cymbra-update-helper: refused, nothing installed: {why}");
    EXIT_REFUSED
}

fn describe(refusal: &HelperRefusal) -> String {
    refusal.to_string()
}

/// Read a whole regular file, refusing anything above `cap` before reading it.
#[cfg(unix)]
fn read_capped(path: &str, cap: u64) -> std::io::Result<Vec<u8>> {
    use std::io::Read as _;
    let file = std::fs::File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > cap {
        return Err(std::io::Error::other(
            "not a regular file under the size cap",
        ));
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    // `take` guards against a file that grows between the stat and the read.
    file.take(cap + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > cap {
        return Err(std::io::Error::other("grew past the size cap"));
    }
    Ok(bytes)
}

/// The installed package version, or `None` when it is not installed.
#[cfg(unix)]
fn installed_version() -> Option<String> {
    let out = std::process::Command::new("dpkg-query")
        .args(["-W", "-f=${Version}", PACKAGE_NAME])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8(out.stdout).ok()?;
    let v = v.trim();
    (!v.is_empty()).then(|| v.to_string())
}

/// Write the verified bytes into a fresh 0700 directory under [`STAGING_ROOT`].
#[cfg(unix)]
fn stage(package: &[u8]) -> std::io::Result<std::path::PathBuf> {
    use std::fs::DirBuilder;
    use std::io::Write as _;
    use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _};

    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(STAGING_ROOT)?;
    let mut nonce = [0u8; 16];
    rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut nonce);
    let name: String = nonce.iter().map(|b| format!("{b:02x}")).collect();
    let dir = std::path::Path::new(STAGING_ROOT).join(format!("update-{name}"));
    // `create` (not `recursive`) fails if it exists: a fresh directory or nothing.
    DirBuilder::new().mode(0o700).create(&dir)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o644)
        .open(dir.join(format!("{PACKAGE_NAME}.deb")))?;
    file.write_all(package)?;
    file.sync_all()?;
    Ok(dir)
}
