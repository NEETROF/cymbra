//! The Lingua-only erasure against live Postgres (change: add-lingua-privacy-controls).
//! `PgDataRepo` is the one place the erasure's SQL exists, and SQL only fails for real
//! against a real server: the transaction, the four deletes and the mark's upsert.
//!
//! Run: `CYMBRA_LINGUA_DATABASE_URL=… cargo test -p cymbra-lingua --test pg_data_it -- --ignored`

use cymbra_lingua::{DataRepo, PgDataRepo};
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

async fn seed(pool: &PgPool, user: uuid::Uuid) {
    sqlx::query(
        "INSERT INTO lingua.word_statuses (user_id, language, lemma, status, updated_at, seq) \
         VALUES ($1, 'en', 'seldom', 'known', 1, nextval('lingua.change_seq'))",
    )
    .bind(user)
    .execute(pool)
    .await
    .expect("seed status");
    sqlx::query(
        "INSERT INTO lingua.declared_levels (user_id, language, level, updated_at, seq) \
         VALUES ($1, 'en', 'B1', 1, nextval('lingua.change_seq'))",
    )
    .bind(user)
    .execute(pool)
    .await
    .expect("seed level");
    sqlx::query(
        "INSERT INTO lingua.cards (user_id, client_id, lemma, updated_at, seq) \
         VALUES ($1, 'seldom', 'seldom', 1, nextval('lingua.change_seq'))",
    )
    .bind(user)
    .execute(pool)
    .await
    .expect("seed card");
    sqlx::query(
        "INSERT INTO lingua.daily_stats (user_id, day, language, device_id, exposures) \
         VALUES ($1, 20000, 'en', 'dev', 3)",
    )
    .bind(user)
    .execute(pool)
    .await
    .expect("seed stat");
}

async fn rows(pool: &PgPool, table: &str, user: uuid::Uuid) -> i64 {
    sqlx::query(&format!(
        "SELECT count(*) AS n FROM {table} WHERE user_id = $1"
    ))
    .bind(user)
    .fetch_one(pool)
    .await
    .expect("count")
    .get::<i64, _>("n")
}

#[tokio::test]
#[ignore = "requires live Postgres with the lingua role"]
async fn erasing_deletes_every_lingua_row_and_records_the_mark() {
    let pool = pool().await;
    let user = uuid::Uuid::now_v7();
    seed(&pool, user).await;
    let repo = PgDataRepo::new(pool.clone());

    let mark = repo.erase(&user.to_string(), 5_000).await.expect("erase");

    assert_eq!(mark, 5_000);
    for table in [
        "lingua.word_statuses",
        "lingua.declared_levels",
        "lingua.cards",
        "lingua.daily_stats",
    ] {
        assert_eq!(rows(&pool, table, user).await, 0, "{table} still has rows");
    }
    assert_eq!(
        repo.erased_at(&user.to_string()).await.expect("mark"),
        5_000
    );
}

#[tokio::test]
#[ignore = "requires live Postgres with the lingua role"]
async fn a_replayed_erasure_keeps_the_later_mark() {
    let pool = pool().await;
    let user = uuid::Uuid::now_v7();
    let repo = PgDataRepo::new(pool.clone());

    assert_eq!(
        repo.erase(&user.to_string(), 9_000).await.expect("first"),
        9_000
    );
    // An out-of-order replay must not move the mark backwards.
    assert_eq!(
        repo.erase(&user.to_string(), 4_000).await.expect("replay"),
        9_000
    );
}

#[tokio::test]
#[ignore = "requires live Postgres with the lingua role"]
async fn a_user_who_never_erased_has_no_mark() {
    let pool = pool().await;
    let repo = PgDataRepo::new(pool);
    let unknown = uuid::Uuid::now_v7().to_string();

    assert_eq!(repo.erased_at(&unknown).await.expect("mark"), 0);
}
