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
//! Spanish-PUD, by `scripts/lingua-data/measure/fr-ud.sh` on UD French-PUD and UD French-GSD's
//! test section (add-lingua-french-forms-tables D9), and by `scripts/lingua-data/measure/it-ud.sh`
//! on UD Italian-PUD and MarkIT's test section (add-lingua-italian-analysis D6).
//!
//! Usage:
//!   lingua-pack-measure [--min-content <percent>] <pack.lingua> <treebank.conllu>
//!
//! Prints the three figures and exits non-zero when any misses its gate. `--min-content` sets the
//! content-word gate apart from the programme's 93.5 %: Italian's is the owner's 90.5 %
//! (add-lingua-italian-forms-tables D8, `measure/it-ud.sh`).

use std::path::Path;
use std::process::ExitCode;

use lingua_core::packs::pack::Pack;
use lingua_pack::measure::{Gates, measure};

const USAGE: &str =
    "usage: lingua-pack-measure [--min-content <percent>] <pack.lingua> <treebank.conllu>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((gates, paths)) = parse(&args) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    match run(Path::new(paths[0]), Path::new(paths[1]), &gates) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("lingua-pack-measure: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The gates and the two paths, or None when the arguments are not `[--min-content P] PACK TREEBANK`.
fn parse(args: &[String]) -> Option<(Gates, [&str; 2])> {
    let mut gates = Gates::default();
    let mut paths = Vec::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--min-content" {
            gates.content = rest.next()?.parse().ok()?;
        } else {
            paths.push(arg.as_str());
        }
    }
    let [pack, treebank] = paths.as_slice() else {
        return None;
    };
    Some((gates, [pack, treebank]))
}

fn run(pack: &Path, treebank: &Path, gates: &Gates) -> Result<bool, Box<dyn std::error::Error>> {
    let pack = Pack::load(&std::fs::read(pack)?)?;
    let m = measure(&pack, &std::fs::read_to_string(treebank)?);
    println!("{}", m.summary());
    let failures = m.failures_against(gates);
    for failure in &failures {
        eprintln!("below the gate: {failure}");
    }
    Ok(failures.is_empty())
}
