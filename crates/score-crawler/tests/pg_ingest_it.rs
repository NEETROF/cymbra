// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License").
// See the workspace root for the full licence text.

//! End-to-end integration test: crawl the local fixture repo, write the corpus,
//! and ingest into a real Postgres `music.catalog_scores`.
//!
//! Skipped unless `CYMBRA_SCORE_DATABASE_URL` is set, e.g. (substitute the
//! backend dev Postgres password from backend/docker-compose.yml):
//!   docker compose -f backend/docker-compose.yml up -d postgres
//!   CYMBRA_SCORE_DATABASE_URL=postgres://cymbra:$PGPASS@localhost:5432/cymbra \
//!     cargo test -p score-crawler --test pg_ingest_it -- --nocapture

use std::path::PathBuf;

use score_crawler::catalog;
use score_crawler::crawl::Orchestrator;
use score_crawler::output::OutputWriter;
use score_crawler::sources::git::GitRepoSource;

#[tokio::test]
async fn crawl_fixture_and_ingest_into_postgres() {
    let Ok(url) = std::env::var("CYMBRA_SCORE_DATABASE_URL") else {
        eprintln!("skip: CYMBRA_SCORE_DATABASE_URL not set");
        return;
    };

    // 1. Crawl the local fixture repo (no network — Orchestrator does not call
    //    prepare()/clone; the git adapter walks the on-disk checkout).
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/musetrainer");
    let adapter = GitRepoSource::musetrainer(fixture);
    let outcome = Orchestrator::new().run(&adapter, None).await;
    assert_eq!(
        outcome.stats.low_confidence, 2,
        "two self-declared-PD fixture scores → low-confidence"
    );

    // 2. Write the corpus (sets each entry's object_key).
    let root = std::env::temp_dir().join("sc_pg_it_corpus");
    let work = std::env::temp_dir().join("sc_pg_it_work");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&work);
    let (_summary, entries) = OutputWriter::new(&root, &work, "safe", "low_confidence")
        .write(&outcome)
        .expect("write corpus");
    assert_eq!(entries.len(), 2);

    // 3. Ingest into a real Postgres catalog.
    let pool = cymbra_music::connect(&url, 2)
        .await
        .expect("connect postgres");
    cymbra_music::MIGRATOR
        .run(&pool)
        .await
        .expect("run migrations");
    let repo = cymbra_music::PgCatalogRepo::new(pool);

    catalog::ingest(&repo, &entries).await.expect("ingest");

    // Every retained score is now in catalog_scores...
    for e in &entries {
        assert!(
            cymbra_music::CatalogRepo::sha_exists(&repo, &e.sha256)
                .await
                .expect("sha_exists"),
            "sha {} present after ingest",
            e.sha256
        );
    }
    // ...and re-ingesting the same content is idempotent (ON CONFLICT sha256).
    let reingested = catalog::ingest(&repo, &entries).await.expect("re-ingest");
    assert_eq!(reingested, 0, "re-ingest inserts no new rows");

    eprintln!(
        "OK: ingested {} scores into music.catalog_scores",
        entries.len()
    );
}
