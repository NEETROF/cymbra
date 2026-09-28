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

//! Postgres integration tests of the Discord adapters (change: add-discord-
//! notifications, tasks 3.2 and 4.4). Require the dev infra up
//! (`backend/docker-compose.yml`) with the roles bootstrapped; the test runs the
//! `jobs` migrations itself, as `worker_svc`.
//!
//! Run: `cargo test -p cymbra-discord --test pg_it -- --ignored`

use cymbra_discord::pg::{PgAnnouncementLedger, enqueue_notify};
use cymbra_discord::{AnnouncementEvent, AnnouncementLedger, Claim};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgPool};

fn url(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"))
}

async fn pool(var: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url(var))
        .await
        .unwrap_or_else(|e| panic!("connect {var}: {e}"))
}

async fn worker() -> PgPool {
    let worker = pool("CYMBRA_WORKER_DATABASE_URL").await;
    cymbra_jobs::MIGRATOR
        .run(&worker)
        .await
        .expect("migrate jobs");
    worker
}

fn event(tag: &str) -> AnnouncementEvent {
    AnnouncementEvent::ScoreAccepted {
        catalog_score_id: tag.to_string(),
    }
}

/// Queued `discord_notify` jobs whose payload names `tag`.
async fn queued(worker: &PgPool, tag: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM jobs.mq_payloads \
         WHERE name = 'discord_notify' AND payload_json->>'catalog_score_id' = $1",
    )
    .bind(tag)
    .fetch_one(worker)
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn the_announcement_exists_iff_the_domain_write_commits() {
    let worker = worker().await;
    let music = pool("CYMBRA_MUSIC_DATABASE_URL").await;

    let rolled_back = uuid::Uuid::new_v4().to_string();
    let mut tx = music.begin().await.unwrap();
    enqueue_notify(&mut tx, &event(&rolled_back)).await;
    tx.rollback().await.unwrap();
    assert_eq!(queued(&worker, &rolled_back).await, 0);

    let committed = uuid::Uuid::new_v4().to_string();
    let mut tx = music.begin().await.unwrap();
    enqueue_notify(&mut tx, &event(&committed)).await;
    tx.commit().await.unwrap();
    assert_eq!(queued(&worker, &committed).await, 1);
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn a_refused_enqueue_never_aborts_the_domain_write() {
    let worker = worker().await;
    // analytics_svc holds no EXECUTE on jobs.enqueue: the enqueue fails.
    let mut conn = sqlx::PgConnection::connect(&url("CYMBRA_ANALYTICS_DATABASE_URL"))
        .await
        .unwrap();
    let tag = uuid::Uuid::new_v4().to_string();
    let mut tx = conn.begin().await.unwrap();
    enqueue_notify(&mut tx, &event(&tag)).await;
    // The transaction is still usable: the failure was confined to its savepoint.
    let one: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(one, 1);
    tx.commit().await.unwrap();
    assert_eq!(queued(&worker, &tag).await, 0);
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn the_ledger_claims_once_and_settles() {
    let ledger = PgAnnouncementLedger::new(worker().await);
    let key = format!("discord:test:{}", uuid::Uuid::new_v4());

    assert_eq!(ledger.claim(&key, 600).await.unwrap(), Claim::Claimed);
    // A second attempt while the first holds it: retry later, do not post.
    assert_eq!(ledger.claim(&key, 600).await.unwrap(), Claim::InFlight);
    ledger
        .mark_posted(std::slice::from_ref(&key))
        .await
        .unwrap();
    assert_eq!(ledger.claim(&key, 600).await.unwrap(), Claim::Settled);
    // Even past any stale window, a posted key is never claimed again.
    assert_eq!(ledger.claim(&key, 0).await.unwrap(), Claim::Settled);
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn a_released_or_abandoned_claim_can_be_taken_again() {
    let ledger = PgAnnouncementLedger::new(worker().await);

    let released = format!("discord:test:{}", uuid::Uuid::new_v4());
    assert_eq!(ledger.claim(&released, 600).await.unwrap(), Claim::Claimed);
    ledger
        .release(std::slice::from_ref(&released))
        .await
        .unwrap();
    assert_eq!(ledger.claim(&released, 600).await.unwrap(), Claim::Claimed);

    // A crash between claim and post: the claim goes stale and is taken over.
    let abandoned = format!("discord:test:{}", uuid::Uuid::new_v4());
    assert_eq!(ledger.claim(&abandoned, 600).await.unwrap(), Claim::Claimed);
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert_eq!(ledger.claim(&abandoned, 0).await.unwrap(), Claim::Claimed);

    let failed = format!("discord:test:{}", uuid::Uuid::new_v4());
    assert_eq!(ledger.claim(&failed, 600).await.unwrap(), Claim::Claimed);
    ledger
        .mark_failed(std::slice::from_ref(&failed), "discord answered 404")
        .await
        .unwrap();
    assert_eq!(ledger.claim(&failed, 0).await.unwrap(), Claim::Settled);
    // release never undoes a settled key.
    ledger.release(std::slice::from_ref(&failed)).await.unwrap();
    assert_eq!(ledger.claim(&failed, 0).await.unwrap(), Claim::Settled);
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn a_batch_claim_wins_only_the_free_keys() {
    let ledger = PgAnnouncementLedger::new(worker().await);
    let key = || format!("discord:test:{}", uuid::Uuid::new_v4());
    let (posted, held, free_a, free_b) = (key(), key(), key(), key());
    assert_eq!(ledger.claim(&posted, 600).await.unwrap(), Claim::Claimed);
    ledger
        .mark_posted(std::slice::from_ref(&posted))
        .await
        .unwrap();
    assert_eq!(ledger.claim(&held, 600).await.unwrap(), Claim::Claimed);

    let all = vec![
        posted.clone(),
        held.clone(),
        free_a.clone(),
        free_b.clone(),
        free_a.clone(),
    ];
    let mut won = ledger.claim_batch(&all, 600).await.unwrap();
    won.sort();
    let mut expected = vec![free_a.clone(), free_b.clone()];
    expected.sort();
    assert_eq!(
        won, expected,
        "settled and live claims are left out, duplicates once"
    );

    // A second job of the same burst wins nothing.
    assert!(ledger.claim_batch(&all, 600).await.unwrap().is_empty());
    // Settling the batch settles each key.
    ledger.mark_posted(&won).await.unwrap();
    assert_eq!(ledger.claim(&free_a, 0).await.unwrap(), Claim::Settled);
    // A stale live claim is taken over by a batch.
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert_eq!(ledger.claim_batch(&all, 0).await.unwrap(), vec![held]);
}
