// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! A card keyed by its studied language, against live Postgres (change:
//! add-lingua-card-language). `PgDeckRepo` is the one place the composite key, the
//! `DEFAULT 'en'` and the `language = ANY(...)` filter exist, and migration 0005's
//! primary-key swap only fails for real against a real server. Since
//! add-lingua-native-language-server a card also carries the language of its gloss
//! (migration 0006, `DEFAULT 'fr'`), a value of the row the winning write sets.
//!
//! Run: `CYMBRA_LINGUA_DATABASE_URL=… cargo test -p cymbra-lingua --test pg_deck_it -- --ignored`

use cymbra_lingua::{Card, DeckRepo, PgDeckRepo};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

async fn pool() -> PgPool {
    let url = std::env::var("CYMBRA_LINGUA_DATABASE_URL").expect("CYMBRA_LINGUA_DATABASE_URL");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect");
    cymbra_lingua::MIGRATOR.run(&pool).await.expect("migrate");
    pool
}

fn card(language: &str, id: &str, gloss: &str, ts: i64) -> Card {
    Card {
        language: language.into(),
        client_id: id.into(),
        lemma: id.into(),
        surface_form: id.into(),
        source_sentence: "…".into(),
        gloss: gloss.into(),
        gloss_language: "fr".into(),
        fsrs_state: "{}".into(),
        deleted: false,
        updated_at: ts,
        device_id: "mac".into(),
        sequence: 0,
    }
}

async fn wipe(pool: &PgPool, user: uuid::Uuid) {
    sqlx::query("DELETE FROM lingua.cards WHERE user_id = $1")
        .bind(user)
        .execute(pool)
        .await
        .expect("wipe");
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn the_primary_key_is_user_language_client_id() {
    let pool = pool().await;
    let cols: Vec<String> = sqlx::query(
        "SELECT a.attname \
         FROM pg_constraint c \
         JOIN pg_class t ON t.oid = c.conrelid \
         JOIN pg_namespace n ON n.oid = t.relnamespace \
         JOIN unnest(c.conkey) WITH ORDINALITY AS k(attnum, ord) ON TRUE \
         JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = k.attnum \
         WHERE n.nspname = 'lingua' AND t.relname = 'cards' AND c.contype = 'p' \
         ORDER BY k.ord",
    )
    .fetch_all(&pool)
    .await
    .expect("pk columns")
    .iter()
    .map(|r| r.get::<String, _>("attname"))
    .collect();
    assert_eq!(cols, ["user_id", "language", "client_id"]);
    // Running the migrator again is a no-op (idempotent swap).
    cymbra_lingua::MIGRATOR
        .run(&pool)
        .await
        .expect("migrate twice");
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn a_row_written_without_a_language_is_english() {
    let pool = pool().await;
    let user = uuid::Uuid::new_v4();
    // What every row written before 0005 looks like: no language column named.
    sqlx::query(
        "INSERT INTO lingua.cards (user_id, client_id, lemma, updated_at, seq) \
         VALUES ($1, 'seldom', 'seldom', 1, nextval('lingua.change_seq'))",
    )
    .bind(user)
    .execute(&pool)
    .await
    .expect("seed legacy card");
    let repo = PgDeckRepo::new(pool.clone());
    let en = repo
        .changes_since(&user.to_string(), 0, &["en".to_string()], false)
        .await
        .expect("pull en");
    assert_eq!(en.len(), 1);
    assert_eq!(en[0].language, "en");
    let es = repo
        .changes_since(&user.to_string(), 0, &["es".to_string()], false)
        .await
        .expect("pull es");
    assert!(es.is_empty());
    wipe(&pool, user).await;
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn the_same_client_id_in_two_languages_is_two_rows_with_independent_lww() {
    let pool = pool().await;
    let user = uuid::Uuid::new_v4();
    let u = user.to_string();
    let repo = PgDeckRepo::new(pool.clone());
    assert!(
        repo.apply_card(&u, &card("en", "son", "fils", 100))
            .await
            .unwrap()
    );
    assert!(
        repo.apply_card(&u, &card("es", "son", "ils sont", 100))
            .await
            .unwrap()
    );
    // A stale Spanish edit loses; a newer English edit wins; neither touches the other.
    assert!(
        !repo
            .apply_card(&u, &card("es", "son", "stale", 50))
            .await
            .unwrap()
    );
    assert!(
        repo.apply_card(&u, &card("en", "son", "fils (edited)", 200))
            .await
            .unwrap()
    );

    let english_only = repo
        .changes_since(&u, 0, &["en".to_string()], false)
        .await
        .unwrap();
    assert_eq!(english_only.len(), 1);
    assert_eq!(english_only[0].gloss, "fils (edited)");

    let both = repo
        .changes_since(&u, 0, &["en".to_string(), "es".to_string()], false)
        .await
        .unwrap();
    assert_eq!(both.len(), 2);
    assert!(both.windows(2).all(|w| w[0].sequence < w[1].sequence));
    let spanish = both.iter().find(|c| c.language == "es").unwrap();
    assert_eq!(spanish.gloss, "ils sont");
    wipe(&pool, user).await;
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn every_stored_gloss_today() {
    // What every row written before 0006 looks like: no gloss language named. The
    // migrator (run by `pool()`) applied 0006 over 0005 and labelled it `fr`.
    let pool = pool().await;
    let user = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO lingua.cards (user_id, client_id, lemma, gloss, updated_at, seq) \
         VALUES ($1, 'seldom', 'seldom', 'rarement', 1, nextval('lingua.change_seq'))",
    )
    .bind(user)
    .execute(&pool)
    .await
    .expect("seed legacy card");
    let repo = PgDeckRepo::new(pool.clone());
    let cards = repo
        .changes_since(&user.to_string(), 0, &["en".to_string()], false)
        .await
        .expect("pull");
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].gloss, "rarement");
    assert_eq!(cards[0].gloss_language, "fr");
    // Running the migrator again is a no-op (ADD COLUMN IF NOT EXISTS).
    cymbra_lingua::MIGRATOR
        .run(&pool)
        .await
        .expect("migrate twice");
    wipe(&pool, user).await;
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn the_gloss_language_travels_with_the_write_that_wins() {
    let pool = pool().await;
    let user = uuid::Uuid::new_v4();
    let u = user.to_string();
    let repo = PgDeckRepo::new(pool.clone());
    let labelled = |gloss: &str, label: &str, ts: i64, device: &str| Card {
        gloss_language: label.into(),
        device_id: device.into(),
        ..card("en", "seldom", gloss, ts)
    };
    // A labelled row round-trips, to a client that reads labels.
    assert!(
        repo.apply_card(&u, &labelled("raramente", "es", 100, "ipad"))
            .await
            .unwrap()
    );
    let stored = repo
        .changes_since(&u, 0, &["en".to_string()], true)
        .await
        .unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].gloss_language, "es");
    // The French-native device writes last: one row, its gloss, its label...
    assert!(
        repo.apply_card(&u, &labelled("rarement", "fr", 200, "mac"))
            .await
            .unwrap()
    );
    // ...and a stale Spanish write afterwards loses, label included.
    assert!(
        !repo
            .apply_card(&u, &labelled("raramente", "es", 150, "ipad"))
            .await
            .unwrap()
    );
    let stored = repo
        .changes_since(&u, 0, &["en".to_string()], false)
        .await
        .unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(
        (stored[0].gloss.as_str(), stored[0].gloss_language.as_str()),
        ("rarement", "fr")
    );
    wipe(&pool, user).await;
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn a_pull_without_any_gloss_language_withholds_a_non_french_gloss() {
    let pool = pool().await;
    let user = uuid::Uuid::new_v4();
    let u = user.to_string();
    let repo = PgDeckRepo::new(pool.clone());
    // A French-glossed card, then an English-glossed one, in the same studied language.
    assert!(
        repo.apply_card(&u, &card("en", "seldom", "rarement", 100))
            .await
            .unwrap()
    );
    assert!(
        repo.apply_card(
            &u,
            &Card {
                gloss_language: "en".into(),
                ..card("en", "rarely", "rarely", 100)
            }
        )
        .await
        .unwrap()
    );
    let en = vec!["en".to_string()];
    // A client that predates labels: the French one only, in the same WHERE as the
    // language filter, so the English one is withheld, not consumed.
    let predates = repo.changes_since(&u, 0, &en, false).await.unwrap();
    assert_eq!(predates.len(), 1);
    assert_eq!(
        (
            predates[0].client_id.as_str(),
            predates[0].gloss_language.as_str()
        ),
        ("seldom", "fr")
    );
    let french_seq = predates[0].sequence;
    // A client that reads labels: both, each with its label, the English one after.
    let reads = repo.changes_since(&u, 0, &en, true).await.unwrap();
    assert_eq!(reads.len(), 2);
    assert_eq!(
        reads
            .iter()
            .map(|c| (c.client_id.as_str(), c.gloss_language.as_str()))
            .collect::<Vec<_>>(),
        [("seldom", "fr"), ("rarely", "en")]
    );
    assert_eq!(
        repo.changes_since(&u, french_seq, &en, true)
            .await
            .unwrap()
            .len(),
        1,
        "from the cursor the old client stopped at, the withheld card is delivered"
    );
    wipe(&pool, user).await;
}
