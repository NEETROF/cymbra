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

//! `lingua-pack-build` — assembles one `pack.lingua` from a manifest and the
//! derived tables, or the same pack as a base and a layer. Invoked by
//! `scripts/lingua-data/build.sh` after it has fetched and reduced the dated sources.
//!
//! Usage:
//!   lingua-pack-build <input-dir> <output.lingua>
//!   lingua-pack-build --studied <studied-dir> <pair-dir> <output.lingua>
//!   lingua-pack-build --split <studied-dir> <pair-dir> <base.lingua> <layer.lingua>
//!
//! With `--studied`, the studied side (`forms.tsv`, `freq.tsv`, `level.tsv`, `grammar.tsv`,
//! `tags.tsv`, `lexical.tsv`) is read from `<studied-dir>` — a committed pair's
//! `tables/<studied>/` — and the rest from `<pair-dir>` (split-lingua-pack-tables-by-language).
//!
//! With `--split`, the two sides are read as with `--studied` and the pack is written as its
//! studied language's base and its pair's layer (split-lingua-pack-format D8): neither file is
//! written when either part fails, and both replace what the paths held only once both are
//! written. The split needs the studied language's pinned tag pool, `tags.tsv`, which every
//! committed language has; the `testdata/` fixtures, one folder holding both sides, have none
//! until `ship-lingua-pack-bases-once` gives them one, and are refused.
//!
//! The input directory holds `manifest.json` (PackMeta + sources),
//! `forms.tsv` (`form<TAB>lemma`), `freq.tsv` (`lemma<TAB>rank`),
//! `gloss.tsv` (`lemma<TAB>gloss`) and `NOTICE`, plus the optional
//! `level.tsv` (`lemma<TAB>A1..C2`) and `mwe.tsv` (`expression<TAB>gloss`).

use std::path::Path;
use std::process::ExitCode;

use lingua_pack::{build_pack, build_split, inputs_from_dirs};

const USAGE: &str = "usage: lingua-pack-build [--studied <studied-dir>] <input-dir> <output.lingua>\n       \
                     lingua-pack-build --split <studied-dir> <pair-dir> <base.lingua> <layer.lingua>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match &args[..] {
        [dir, out] => whole(Path::new(dir), Path::new(dir), out),
        [flag, studied, dir, out] if flag == "--studied" => {
            whole(Path::new(studied), Path::new(dir), out)
        }
        [flag, studied, dir, base, layer] if flag == "--split" => {
            split(Path::new(studied), Path::new(dir), base, layer)
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    match result {
        Ok(written) => {
            println!("{written}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("lingua-pack-build: {e}");
            ExitCode::FAILURE
        }
    }
}

fn whole(studied: &Path, dir: &Path, out: &str) -> Result<String, Box<dyn std::error::Error>> {
    let inputs = inputs_from_dirs(studied, dir)?;
    let bytes = build_pack(&inputs)?;
    std::fs::write(out, &bytes)?;
    Ok(format!("wrote {out} ({} bytes)", bytes.len()))
}

/// Both parts are built before either is written, so a part that fails leaves no file; each is
/// written beside its path first, and both are moved into place once both are written, so a
/// write that fails never leaves a new base beside an old layer.
fn split(
    studied: &Path,
    dir: &Path,
    base: &str,
    layer: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let inputs = inputs_from_dirs(studied, dir)?;
    let parts = build_split(&inputs)?;
    let staged = |path: &str| format!("{path}.tmp");
    let written = std::fs::write(staged(base), &parts.base)
        .and_then(|()| std::fs::write(staged(layer), &parts.layer));
    if let Err(e) = written {
        let _ = std::fs::remove_file(staged(base));
        let _ = std::fs::remove_file(staged(layer));
        return Err(e.into());
    }
    std::fs::rename(staged(base), base)?;
    std::fs::rename(staged(layer), layer)?;
    Ok(format!(
        "wrote {base} ({} bytes) and {layer} ({} bytes)",
        parts.base.len(),
        parts.layer.len()
    ))
}
