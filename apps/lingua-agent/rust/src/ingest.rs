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

//! Ingestion: assistant text → lemmas → exposure counters. The spec invariant is that
//! **no transcript content is persisted** — only lemmas, counts and timestamps. Proper
//! nouns out of the lexicon are not counted (they are not vocabulary). Idempotence is
//! keyed on the transcript byte offset, so a re-run of the `Stop` hook over a growing
//! transcript only ingests the new turns.

use lingua_core::analysis::percent::TokenClass;
use lingua_core::packs::Pack;

use crate::engine::analyse;
use crate::source::SessionSource;
use crate::store::Store;

/// Records exposures for every countable lemma in `texts`. Returns the number of
/// occurrences counted. Pure over the store (unit-tested with an in-memory store).
pub fn ingest_texts(
    store: &Store,
    pack: &Pack,
    texts: &[String],
    source: &str,
    timestamp: i64,
) -> rusqlite::Result<usize> {
    let knowledge = store.knowledge_state()?;
    let mut counted = 0;
    for text in texts {
        let analysis = analyse(pack, &knowledge, text);
        for token in &analysis.tokens {
            if token.class == TokenClass::ProperNounOutOfLexicon {
                continue;
            }
            store.record_exposure(&token.lemma, 1, source, timestamp)?;
            counted += 1;
        }
    }
    Ok(counted)
}

/// Ingest the new turns of a transcript through a source, advancing its offset. Returns
/// the number of occurrences counted this run (0 when there is nothing new).
pub fn run_ingest(
    store: &Store,
    pack: &Pack,
    source: &dyn SessionSource,
    transcript_id: &str,
    timestamp: i64,
) -> std::io::Result<usize> {
    let offset = store.ingest_offset(transcript_id).unwrap_or(0);
    let (new_offset, texts) = source.extract(offset)?;
    let counted = ingest_texts(store, pack, &texts, transcript_id, timestamp).unwrap_or(0);
    let _ = store.set_ingest_offset(transcript_id, new_offset);
    Ok(counted)
}
