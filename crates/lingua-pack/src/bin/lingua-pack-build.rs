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
//!
//! The input directory holds `manifest.json` (PackMeta + sources),
//! `forms.tsv` (`form<TAB>lemma`), `freq.tsv` (`lemma<TAB>rank`),
//! `gloss.tsv` (`lemma<TAB>gloss`) and `NOTICE`.

use std::path::Path;
use std::process::ExitCode;

use lingua_pack::{build_pack, inputs_from_dir};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: lingua-pack-build <input-dir> <output.lingua>");
        return ExitCode::FAILURE;
    }
    match run(Path::new(&args[1]), Path::new(&args[2])) {
        Ok(size) => {
            println!("wrote {} ({size} bytes)", args[2]);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("lingua-pack-build: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(dir: &Path, out: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let inputs = inputs_from_dir(dir)?;
    let bytes = build_pack(&inputs)?;
    std::fs::write(out, &bytes)?;
    Ok(bytes.len())
}
