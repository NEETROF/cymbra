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
//! derived tables. Invoked by `scripts/lingua-data/build.sh` after it has
//! fetched and reduced the dated sources.
//!
//! Usage:
//!   lingua-pack-build <input-dir> <output.lingua>
//!   lingua-pack-build --studied <studied-dir> <pair-dir> <output.lingua>
//!
//! With `--studied`, the studied side (`forms.tsv`, `freq.tsv`, `level.tsv`, `grammar.tsv`,
//! `tags.tsv`, `lexical.tsv`) is read from `<studied-dir>` — a committed pair's
//! `tables/<studied>/` — and the rest from `<pair-dir>` (split-lingua-pack-tables-by-language).
//!
//! The input directory holds `manifest.json` (PackMeta + sources),
//! `forms.tsv` (`form<TAB>lemma`), `freq.tsv` (`lemma<TAB>rank`),
//! `gloss.tsv` (`lemma<TAB>gloss`) and `NOTICE`, plus the optional
//! `level.tsv` (`lemma<TAB>A1..C2`) and `mwe.tsv` (`expression<TAB>gloss`).

use std::path::Path;
use std::process::ExitCode;

use lingua_pack::{build_pack, inputs_from_dirs};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (studied, dir, out) = match &args[..] {
        [dir, out] => (dir, dir, out),
        [flag, studied, dir, out] if flag == "--studied" => (studied, dir, out),
        _ => {
            eprintln!(
                "usage: lingua-pack-build [--studied <studied-dir>] <input-dir> <output.lingua>"
            );
            return ExitCode::FAILURE;
        }
    };
    match run(Path::new(studied), Path::new(dir), Path::new(out)) {
        Ok(size) => {
            println!("wrote {out} ({size} bytes)");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("lingua-pack-build: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(studied: &Path, dir: &Path, out: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let inputs = inputs_from_dirs(studied, dir)?;
    let bytes = build_pack(&inputs)?;
    std::fs::write(out, &bytes)?;
    Ok(bytes.len())
}
