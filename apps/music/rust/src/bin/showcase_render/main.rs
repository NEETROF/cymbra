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

//! `showcase-render <log> <soundfonts-dir> <out.wav>` — replays the showcase
//! video's sound log through the app's own mixer into a WAV file (change:
//! add-site-music-showcase, D6). Run by `tool/record_showcase_video.sh`.

use anyhow::{Context, Result, bail};
use rust_lib_music::showcase;
use std::path::Path;

const SAMPLE_RATE: u32 = 48_000;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [log, soundfonts, out] = args.as_slice() else {
        bail!("usage: showcase-render <log> <soundfonts-dir> <out.wav>");
    };
    let text = std::fs::read_to_string(log).with_context(|| format!("read {log}"))?;
    let events = showcase::parse_log(&text)?;
    let (left, right) = showcase::render(&events, Path::new(soundfonts), SAMPLE_RATE)?;
    std::fs::write(out, showcase::encode_wav(&left, &right, SAMPLE_RATE))
        .with_context(|| format!("write {out}"))?;
    eprintln!(
        "wrote {out}: {:.2} s, {} commands",
        left.len() as f64 / f64::from(SAMPLE_RATE),
        events.len()
    );
    Ok(())
}
