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

//! Italian's detection guard, measured (add-lingua-italian-detection-guard, task 1.1): reads a
//! file of blocks, one per line, and writes for each whichlang's class and whether the core reads
//! it as Italian — `block_is_studied(block, Italian)`, the guard included — as `ita\t1`.
//! `scripts/lingua-data/measure/it_guard.py` runs it.
//!
//! With `--time`, it times instead, over every block, best of seven passes: whichlang alone; the
//! gate for English, which reads the block as the gate does and has no guard; and the gate for
//! Italian. The guard's cost is the difference between the two gates, in µs per KB of text (D5).
//!
//! ```bash
//! cargo run --release -p lingua-core --example italian_guard_measure -- blocks.txt
//! cargo run --release -p lingua-core --example italian_guard_measure -- --time blocks.txt
//! ```

use std::hint::black_box;
use std::io::{BufWriter, Write};
use std::time::Instant;

use lingua_core::analysis::language::{StudiedLanguage, block_is_studied};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (time, path) = match args.as_slice() {
        [flag, path] if flag == "--time" => (true, path),
        [path] => (false, path),
        _ => {
            eprintln!("usage: italian_guard_measure [--time] <blocks.txt>");
            std::process::exit(2);
        }
    };
    let text = std::fs::read_to_string(path)?;
    let blocks: Vec<&str> = text.lines().collect();
    if time {
        let kb = text.len() as f64 / 1024.0;
        let best = |pass: &dyn Fn()| {
            (0..7)
                .map(|_| {
                    let start = Instant::now();
                    pass();
                    start.elapsed().as_secs_f64()
                })
                .fold(f64::INFINITY, f64::min)
        };
        let whichlang = best(&|| {
            for block in &blocks {
                black_box(whichlang::detect_language(black_box(block)));
            }
        });
        let gate = |language: StudiedLanguage| {
            best(&|| {
                for block in &blocks {
                    black_box(block_is_studied(black_box(block), language));
                }
            })
        };
        let (english, italian) = (
            gate(StudiedLanguage::English),
            gate(StudiedLanguage::Italian),
        );
        println!(
            "{} blocks, {kb:.0} KB: whichlang {:.2} µs/KB, English's gate {:.2}, Italian's {:.2}; \
             the guard {:.2} µs/KB ({:.0} % of whichlang)",
            blocks.len(),
            1e6 * whichlang / kb,
            1e6 * english / kb,
            1e6 * italian / kb,
            1e6 * (italian - english) / kb,
            100.0 * (italian - english) / whichlang
        );
        return Ok(());
    }
    let mut out = BufWriter::new(std::io::stdout().lock());
    for block in blocks {
        let class = whichlang::detect_language(block).three_letter_code();
        let italian = block_is_studied(block, StudiedLanguage::Italian);
        writeln!(out, "{class}\t{}", u8::from(italian))?;
    }
    out.flush()
}
