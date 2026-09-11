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

//! `lingua-pack-manifest` — emits the committed pack registry that the Lingua ops
//! console serves (change: add-lingua-back-office, D5). For each input directory it
//! builds the pack in memory and records `{studied, native, pack_version,
//! analyzer_version, built_at, size_bytes, notice}`. `built_at` is preserved when a
//! pack's content (version + size + NOTICE) is unchanged, so a reproducible rebuild
//! leaves an empty diff. Invoked by `scripts/lingua-data/build.sh emit-manifest`.
//!
//! Usage:
//!   lingua-pack-manifest [--check] --built-at &lt;yyyy-mm-dd&gt; &lt;manifest.json&gt; &lt;input-dir&gt;...
//!
//! `--check` rebuilds and compares the committed manifest by content (ignoring
//! `built_at`); a stale manifest exits non-zero instead of being rewritten.

use std::path::Path;
use std::process::ExitCode;

use lingua_pack::{build_pack, inputs_from_dir};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
struct Entry {
    studied: String,
    native: String,
    pack_version: String,
    analyzer_version: String,
    built_at: String,
    size_bytes: i64,
    notice: String,
}

impl Entry {
    /// Everything but the build date — what "the pack has not changed" means.
    fn same_content(&self, o: &Entry) -> bool {
        self.studied == o.studied
            && self.native == o.native
            && self.pack_version == o.pack_version
            && self.analyzer_version == o.analyzer_version
            && self.size_bytes == o.size_bytes
            && self.notice == o.notice
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Manifest {
    packs: Vec<Entry>,
}

fn build_entry(dir: &Path, built_at: &str) -> Result<Entry, Box<dyn std::error::Error>> {
    let inputs = inputs_from_dir(dir)?;
    let bytes = build_pack(&inputs)?;
    Ok(Entry {
        studied: inputs.meta.studied,
        native: inputs.meta.native,
        pack_version: inputs.meta.pack_version,
        analyzer_version: inputs.meta.analyzer_version,
        built_at: built_at.to_string(),
        size_bytes: bytes.len() as i64,
        notice: inputs.notice,
    })
}

fn read_committed(path: &Path) -> Manifest {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
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
    let (out, dirs) = positional
        .split_first()
        .ok_or("usage: lingua-pack-manifest [--check] --built-at <yyyy-mm-dd> <manifest.json> <input-dir>...")?;
    if dirs.is_empty() {
        return Err("at least one input directory is required".into());
    }
    // A stable placeholder is enough for --check (it ignores built_at); emit needs a real date.
    let built_at = built_at.unwrap_or_else(|| "0000-00-00".to_string());

    let committed = read_committed(Path::new(out));
    let mut packs = Vec::new();
    for dir in dirs {
        let mut e = build_entry(Path::new(dir), &built_at)?;
        // Preserve the recorded build date when the pack's content is unchanged.
        if let Some(prev) = committed.packs.iter().find(|p| e.same_content(p)) {
            e.built_at = prev.built_at.clone();
        }
        packs.push(e);
    }
    packs.sort_by(|a, b| (&a.studied, &a.native).cmp(&(&b.studied, &b.native)));

    if check {
        let content_matches = packs.len() == committed.packs.len()
            && packs
                .iter()
                .all(|e| committed.packs.iter().any(|c| e.same_content(c)));
        if !content_matches {
            return Err(format!(
                "packs-manifest.json is stale relative to the built packs — re-run \
                 `scripts/lingua-data/build.sh emit-manifest` and commit {out}"
            )
            .into());
        }
        return Ok(());
    }

    let json = serde_json::to_string_pretty(&Manifest { packs })? + "\n";
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
