// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The read-only data-pack registry served by `AdminListDataPacks` (change:
//! add-lingua-back-office, D5). The manifest is produced by the pack pipeline
//! (`scripts/lingua-data/build.sh emit-manifest`) and embedded at compile time; it is
//! informational (distribution stays bundled in the extension in v1) and paves the way
//! for a future OTA, when this source moves to a store without changing shape.

use cymbra_platform::{AppError, Result};
use serde::Deserialize;

/// One published pack — metadata only, never account data.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DataPack {
    pub studied: String,
    pub native: String,
    pub pack_version: String,
    pub analyzer_version: String,
    pub built_at: String,
    pub size_bytes: i64,
    pub notice: String,
}

#[derive(Deserialize)]
struct Manifest {
    packs: Vec<DataPack>,
}

/// The committed registry, embedded at compile time.
const MANIFEST_JSON: &str = include_str!("../packs-manifest.json");

/// Parse a registry manifest; an invalid document is an internal error.
pub fn parse(json: &str) -> Result<Vec<DataPack>> {
    let m: Manifest = serde_json::from_str(json)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid packs manifest: {e}")))?;
    Ok(m.packs)
}

/// The embedded registry. Parsed once at module construction so an invalid committed
/// manifest fails the boot rather than a request.
pub fn embedded() -> Result<Vec<DataPack>> {
    parse(MANIFEST_JSON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_committed_manifest_parses() {
        let packs = embedded().expect("committed packs-manifest.json must parse");
        assert!(
            !packs.is_empty(),
            "the registry should list at least one pack"
        );
        // Every entry is complete metadata, with no account-shaped field to leak.
        for p in &packs {
            assert!(!p.studied.is_empty() && !p.native.is_empty());
            assert!(!p.pack_version.is_empty() && !p.analyzer_version.is_empty());
            assert!(p.size_bytes > 0);
        }
    }

    #[test]
    fn a_malformed_manifest_is_an_internal_error() {
        assert!(matches!(parse("{ not json"), Err(AppError::Internal(_))));
        assert!(matches!(
            parse(r#"{"packs":"nope"}"#),
            Err(AppError::Internal(_))
        ));
    }

    #[test]
    fn a_well_formed_fixture_round_trips() {
        let json = r#"{"packs":[
            {"studied":"en","native":"fr","pack_version":"1.2.3","analyzer_version":"1.0.0",
             "built_at":"2026-09-11","size_bytes":4096,"notice":"AGID; wordfreq; kaikki."}
        ]}"#;
        let packs = parse(json).unwrap();
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].pack_version, "1.2.3");
        assert_eq!(packs[0].size_bytes, 4096);
    }
}
