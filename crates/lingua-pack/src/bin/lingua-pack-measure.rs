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

//! `lingua-pack-measure` — a built pack's lemmas against a treebank, held to the programme's gates
//! (add-lingua-spanish-forms-tables D5). Run by `scripts/lingua-data/measure/es-pud.sh` on UD
//! Spanish-PUD, and by `scripts/lingua-data/measure/fr-ud.sh` on UD French-PUD and UD French-GSD's
//! test section (add-lingua-french-forms-tables D9).
//!
//! Usage:
//!   lingua-pack-measure <pack.lingua> <treebank.conllu>
//!
//! Prints the three figures and exits non-zero when any misses its gate.

use std::path::Path;
use std::process::ExitCode;

use lingua_core::packs::pack::Pack;
use lingua_pack::measure::measure;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: lingua-pack-measure <pack.lingua> <treebank.conllu>");
        return ExitCode::FAILURE;
    }
    match run(Path::new(&args[1]), Path::new(&args[2])) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("lingua-pack-measure: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(pack: &Path, treebank: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let pack = Pack::load(&std::fs::read(pack)?)?;
    let m = measure(&pack, &std::fs::read_to_string(treebank)?);
    println!("{}", m.summary());
    let failures = m.failures();
    for failure in &failures {
        eprintln!("below the gate: {failure}");
    }
    Ok(failures.is_empty())
}
