// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License").
// See the workspace root for the full licence text.

//! Full-pipeline integration test through the **Docker** converter backend:
//! crawl the Mutopia `.ly` fixtures and convert them to `.mxl` inside a
//! `python-ly` image — no converter installed on the host.
//!
//! Skipped unless `SCORE_CRAWLER_LY_IMAGE` names a Docker image carrying `ly`
//! (python-ly) on PATH, e.g.:
//!   printf 'FROM python:3.12-slim\nRUN pip install python-ly\n' > Dockerfile
//!   docker build -t score-crawler-ly .
//!   SCORE_CRAWLER_LY_IMAGE=score-crawler-ly \
//!     cargo test -p score-crawler --test docker_convert_it -- --nocapture

use std::path::PathBuf;

use score_crawler::convert::{ConversionStatus, ConverterBackend, Converters, init_converters};
use score_crawler::crawl::Orchestrator;
use score_crawler::sources::mutopia::MutopiaSource;

#[tokio::test]
async fn mutopia_ly_converts_to_mxl_via_docker() {
    let Ok(image) = std::env::var("SCORE_CRAWLER_LY_IMAGE") else {
        eprintln!("skip: set SCORE_CRAWLER_LY_IMAGE=<python-ly docker image>");
        return;
    };

    // Route conversion through Docker + the python-ly image.
    init_converters(Converters {
        backend: ConverterBackend::Docker,
        lilypond_image: image,
        ..Converters::default()
    });

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutopia");
    let out = Orchestrator::new()
        .run(&MutopiaSource::new(fixture), None)
        .await;
    eprintln!("stats: {:?}", out.stats);

    // The CC-BY-SA and Public-Domain files pass the gate and are converted to a
    // valid .mxl by python-ly running in Docker; the CC-BY-NC file is rejected.
    assert_eq!(out.stats.accepted, 2, "two free .ly converted via docker");
    assert_eq!(
        out.stats.rejected, 1,
        "the CC-BY-NC file is licence-rejected"
    );
    assert!(
        out.prepared
            .iter()
            .all(|p| p.entry.conversion_status == ConversionStatus::Converted)
    );
    for p in &out.prepared {
        assert!(
            cymbra_musicxml_core::mxl::is_mxl(&p.mxl),
            "produced a real .mxl container"
        );
    }
    eprintln!(
        "OK: {} Mutopia .ly converted to .mxl via Docker",
        out.prepared.len()
    );
}
