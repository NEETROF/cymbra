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

//! `lingua-pack-manifest` — emits (or verifies) the committed pack registry the Lingua
//! ops console serves (change: add-lingua-back-office, D5). Thin CLI: it builds each
//! pack, reads the committed manifest, and delegates the merge / staleness logic to
//! `lingua_pack::manifest` (host-tested there). Invoked by
//! `scripts/lingua-data/build.sh emit-manifest`.
//!
//! Usage:
//!   lingua-pack-manifest [--check] --built-at <yyyy-mm-dd> <manifest.json> <input-dir>...
//!
//! `--check` rebuilds and compares the committed manifest by content (ignoring
//! `built_at`); a stale manifest exits non-zero instead of being rewritten.

use std::path::Path;
use std::process::ExitCode;

use lingua_pack::manifest::{PackManifest, PackManifestEntry, is_stale, merge};
use lingua_pack::{build_pack, inputs_from_dir};

/// Build one pack in memory and record its registry entry.
fn build_entry(
    dir: &Path,
    built_at: &str,
) -> Result<PackManifestEntry, Box<dyn std::error::Error>> {
    let inputs = inputs_from_dir(dir)?;
    let bytes = build_pack(&inputs)?;
    Ok(PackManifestEntry {
        studied: inputs.meta.studied,
        native: inputs.meta.native,
        pack_version: inputs.meta.pack_version,
        analyzer_version: inputs.meta.analyzer_version,
        built_at: built_at.to_string(),
        size_bytes: bytes.len() as i64,
        notice: inputs.notice,
    })
}

fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut check = false;
    let mut built_at = None;
    let mut positional = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--check" => check = true,
            "--built-at" => built_at = it.next().cloned(),
            _ => positional.push(a.clone()),
        }
    }
    let (out, dirs) = positional.split_first().ok_or(
        "usage: lingua-pack-manifest [--check] --built-at <yyyy-mm-dd> <manifest.json> <input-dir>...",
    )?;
    if dirs.is_empty() {
        return Err("at least one input directory is required".into());
    }
    // A stable placeholder is enough for --check (it ignores built_at); emit needs a real date.
    let built_at = built_at.unwrap_or_else(|| "0000-00-00".to_string());

    let committed: PackManifest = std::fs::read_to_string(out)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let fresh = dirs
        .iter()
        .map(|d| build_entry(Path::new(d), &built_at))
        .collect::<Result<Vec<_>, _>>()?;

    if check {
        if is_stale(&fresh, &committed) {
            return Err(format!(
                "packs-manifest.json is stale relative to the built packs — re-run \
                 `scripts/lingua-data/build.sh emit-manifest` and commit {out}"
            )
            .into());
        }
        return Ok(());
    }

    let packs = merge(fresh, &committed);
    let json = serde_json::to_string_pretty(&PackManifest { packs })? + "\n";
    std::fs::write(out, json)?;
    println!("wrote {out}");
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lingua-pack-manifest: {e}");
            ExitCode::FAILURE
        }
    }
}
