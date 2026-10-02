//! The ops aggregate's contributor counts against live Postgres (change:
//! add-discord-notifications, task 3.8): each summed figure carries the number of
//! accounts that contributed to it, so a published figure can be gated on them.
//!
//! Run: `CYMBRA_LINGUA_DATABASE_URL=… cargo test -p cymbra-lingua --test pg_admin_usage_it -- --ignored`

use cymbra_lingua::{LinguaAdminRepo, PgLinguaAdminRepo};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with the lingua role"]
async fn each_figure_counts_the_accounts_behind_it() {
    let url = std::env::var("CYMBRA_LINGUA_DATABASE_URL").expect("CYMBRA_LINGUA_DATABASE_URL");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect");
    cymbra_lingua::MIGRATOR.run(&pool).await.expect("migrate");

    // A window of our own, far from any other test's rows.
    let day: i32 = 90_000 + (uuid::Uuid::new_v4().as_u128() % 1_000) as i32;
    // Five active accounts: all read, two learned, one reviewed.
    for i in 0..5 {
        sqlx::query(
            "INSERT INTO lingua.daily_stats \
             (user_id, day, language, device_id, exposures, words_learned, reviews_done) \
             VALUES ($1, $2, 'en', 'd', 100, $3, $4)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(day)
        .bind(if i < 2 { 3 } else { 0 })
        .bind(if i == 0 { 7 } else { 0 })
        .execute(&pool)
        .await
        .expect("seed");
    }

    let usage = PgLinguaAdminRepo::new(pool)
        .usage(day, day)
        .await
        .expect("usage");
    assert_eq!(usage.active_accounts, 5);
    assert_eq!((usage.words_read, usage.readers), (500, 5));
    assert_eq!((usage.words_learned, usage.learners), (6, 2));
    assert_eq!((usage.reviews, usage.reviewers), (7, 1));
}
