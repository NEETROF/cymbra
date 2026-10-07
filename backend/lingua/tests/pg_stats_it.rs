// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! A daily statistic's native language, against live Postgres (change:
//! add-lingua-native-language-server). `PgStatsRepo` is the one place the column, its
//! `DEFAULT 'fr'` and the upsert that sets it exist, and migration 0006 only applies for
//! real, over 0001-0005, against a real server. This database is fresh (CI boots one):
//! every row here is written after 0006, so a row "written before it" is one written by
//! SQL that names no label, and what that proves is the column's default, not a rewrite.
//!
//! Run: `CYMBRA_LINGUA_DATABASE_URL=… cargo test -p cymbra-lingua --test pg_stats_it -- --ignored`

use cymbra_lingua::{DailyStat, PgStatsRepo, StatsRepo};
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

fn stat(day: i32, device: &str, native_language: &str, reviews: u32) -> DailyStat {
    DailyStat {
        day,
        language: "en".into(),
        device_id: device.into(),
        exposures: 0,
        words_learned: 0,
        reviews_done: reviews,
        unknown_seen: Some(0),
        native_language: native_language.into(),
    }
}

async fn wipe(pool: &PgPool, user: uuid::Uuid) {
    sqlx::query("DELETE FROM lingua.daily_stats WHERE user_id = $1")
        .bind(user)
        .execute(pool)
        .await
        .expect("wipe");
}

/// The two label columns as the catalogue describes them: (nullable, default), in the
/// order `cards.gloss_language`, `daily_stats.native_language`.
async fn label_columns(pool: &PgPool) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    for (table, column) in [
        ("cards", "gloss_language"),
        ("daily_stats", "native_language"),
    ] {
        let row = sqlx::query(
            "SELECT is_nullable, column_default FROM information_schema.columns \
             WHERE table_schema = 'lingua' AND table_name = $1 AND column_name = $2",
        )
        .bind(table)
        .bind(column)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("lingua.{table}.{column}: {e}"));
        out.push((
            row.get::<String, _>("is_nullable"),
            row.get::<Option<String>, _>("column_default"),
        ));
    }
    out
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn migration_0006_adds_two_defaulted_columns_and_its_sql_applies_twice() {
    let pool = pool().await;
    let applied = label_columns(&pool).await;
    let defaulted_fr = ("NO".to_owned(), Some("'fr'::text".to_owned()));
    assert_eq!(applied, vec![defaulted_fr.clone(), defaulted_fr]);
    // Running the migrator again proves nothing about `IF NOT EXISTS`: the ledger says
    // 0006 is applied, and it is skipped. Its own SQL, run raw a second time, is what
    // must be a no-op — and leave the columns as they were.
    let sql = &cymbra_lingua::MIGRATOR
        .iter()
        .find(|m| m.version == 6)
        .expect("migration 0006 in the migrator")
        .sql;
    sqlx::raw_sql(sql)
        .execute(&pool)
        .await
        .expect("0006's SQL applies twice");
    assert_eq!(label_columns(&pool).await, applied);
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn a_row_written_without_a_native_language_is_french() {
    let pool = pool().await;
    let user = uuid::Uuid::new_v4();
    // `pool()` ran 0006 before any row existed, so this row is written AFTER the
    // migration, by SQL that names no native language — as every client's upsert did
    // before the field. What it proves is the column's default: the value a row that
    // predates 0006 also reads, by Postgres's catalogue default, without a rewrite. That
    // a real pre-0006 row reads so is rehearsed on a copy of production (task 5.1).
    sqlx::query(
        "INSERT INTO lingua.daily_stats \
           (user_id, day, language, device_id, exposures, unknown_seen) \
         VALUES ($1, 20000, 'en', 'mac', 3, 1)",
    )
    .bind(user)
    .execute(&pool)
    .await
    .expect("seed legacy stat");
    let repo = PgStatsRepo::new(pool.clone());
    let rows = repo
        .range(&user.to_string(), 20_000, 20_000, None)
        .await
        .expect("range");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].native_language, "fr");
    assert_eq!(rows[0].exposures, 3);
    wipe(&pool, user).await;
}

#[tokio::test]
#[ignore = "needs CYMBRA_LINGUA_DATABASE_URL"]
async fn the_last_upsert_of_a_day_sets_its_native_language_and_its_counts() {
    let pool = pool().await;
    let user = uuid::Uuid::new_v4();
    let u = user.to_string();
    let repo = PgStatsRepo::new(pool.clone());
    // A labelled row round-trips.
    repo.upsert(&u, &stat(20_000, "mac", "es", 5))
        .await
        .unwrap();
    let rows = repo.range(&u, 20_000, 20_000, Some("en")).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].native_language, "es");
    // The native language changes during the day: the last upsert sets the row.
    repo.upsert(&u, &stat(20_000, "mac", "fr", 9))
        .await
        .unwrap();
    let rows = repo.range(&u, 20_000, 20_000, Some("en")).await.unwrap();
    assert_eq!(rows.len(), 1, "a value of the row, not of its key");
    assert_eq!(
        (rows[0].native_language.as_str(), rows[0].reviews_done),
        ("fr", 9)
    );
    wipe(&pool, user).await;
}
