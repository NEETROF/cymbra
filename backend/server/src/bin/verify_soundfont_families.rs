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

//! `verify-soundfont-families` — the one-shot ops verification pass over the
//! existing `music.soundfonts` rows (change: add-drum-audio-channel, task 4.5).
//!
//! Every row predates the upload-time family verification, so its recorded
//! family is an uploader's claim. This command re-reads each stored `.sf2`
//! object, checks the recorded family against the file's actual preset banks
//! (the same `verify_declared_family` rule the upload boundary enforces), and
//! **reports** mismatches — it never rewrites anything. Read-only by
//! construction: fixing a bad row is a human decision.
//!
//! Reuses the SAME env/config as `cymbra-server` (music DB read + the private
//! SoundFont bucket).
//!
//! Usage:
//!     cargo run -p cymbra-server --bin verify-soundfont-families

use anyhow::{Context, Result};
use cymbra_music::{PgSoundFontRepo, SoundFontRepo, normalize_family, verify_declared_family};
use cymbra_platform::config::Config;
use cymbra_server::maintenance::soundfont_object_store;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    if std::env::args().skip(1).any(|a| a == "-h" || a == "--help") {
        println!(
            "verify-soundfont-families\n\
             \n  Read-only: re-reads every music.soundfonts object and reports any row\n\
             \n  whose recorded family its preset banks cannot support. Never rewrites."
        );
        return Ok(());
    }

    let cfg = Config::from_env()?;
    let db_url = cfg
        .music_database_url
        .as_deref()
        .context("CYMBRA_MUSIC_DATABASE_URL is required for the verification pass")?;
    let pool = cymbra_music::connect(db_url, 2)
        .await
        .context("connecting to the music database")?;
    let repo = PgSoundFontRepo::new(pool);
    let store = soundfont_object_store(&cfg)?;

    let fonts = repo.list().await.context("listing the soundfont catalog")?;
    let mut ok = 0usize;
    let mut mismatched = 0usize;
    let mut unreadable = 0usize;
    for font in &fonts {
        let bytes = match store.get(&font.object_key).await {
            Ok(b) => b,
            Err(e) => {
                unreadable += 1;
                println!(
                    "UNREADABLE  {} ({}): object {} not readable: {e}",
                    font.id, font.instrument, font.object_key
                );
                continue;
            }
        };
        // Normalise so a not-yet-migrated `piano` row is judged as `keyboard`
        // (the same bridge the upload boundary applies).
        match verify_declared_family(&normalize_family(&font.instrument), &bytes) {
            Ok(()) => ok += 1,
            Err(refusal) => {
                mismatched += 1;
                println!(
                    "MISMATCH    {} (recorded '{}', {} bytes): {}",
                    font.id,
                    font.instrument,
                    bytes.len(),
                    refusal.message()
                );
            }
        }
    }
    println!(
        "Verified {} font(s): {ok} ok, {mismatched} mismatched, {unreadable} unreadable. \
         Nothing was rewritten.",
        fonts.len()
    );
    Ok(())
}
