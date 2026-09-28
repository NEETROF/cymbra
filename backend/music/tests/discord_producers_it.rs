//! The Discord producers against Postgres (change: add-discord-notifications, tasks
//! 4.1–4.4): accepting a SoundFont or a score enqueues `discord_notify` in the
//! status write's transaction, and the season-best upsert enqueues an anonymous one
//! only when a record held by ANOTHER player is beaten. Requires
//! the dev infra up (`backend/docker-compose.yml`) with the roles bootstrapped; the
//! test runs the `jobs` (as `worker_svc`) and `music` (as `music_svc`) migrations.
//!
//! Run: `cargo test -p cymbra-music --test discord_producers_it -- --ignored`

use cymbra_music::{
    CatalogSearchRepo, GlobalLeaderboardRepo, GlobalSeasonBest, Mode, PgCatalogSearchRepo,
    PgGlobalLeaderboardRepo, PgSoundFontRepo, SoundFontRepo,
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

async fn pool(var: &str) -> PgPool {
    let url = std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"));
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap_or_else(|e| panic!("connect {var}: {e}"))
}

/// Migrated `(worker_svc, music_svc)` pools.
async fn pools() -> (PgPool, PgPool) {
    let worker = pool("CYMBRA_WORKER_DATABASE_URL").await;
    cymbra_jobs::MIGRATOR
        .run(&worker)
        .await
        .expect("migrate jobs");
    let music = pool("CYMBRA_MUSIC_DATABASE_URL").await;
    cymbra_music::MIGRATOR
        .run(&music)
        .await
        .expect("migrate music");
    (worker, music)
}

/// Seed a catalog piece with `status`; returns its id.
async fn seed_piece(music: &PgPool, status: &str) -> String {
    let piece = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO music.catalog_scores (id, source, source_url, source_item_id, license, \
            confidence, sha256, origin_format, conversion_status, object_key, work_key, \
            moderation_status, title) \
         VALUES ($1, 'test', 'https://example.test', $2, 'CC0-1.0', 'verified', $2, 'mxl', \
            'converted', $2, $2, $3, 'Test piece')",
    )
    .bind(piece)
    .bind(piece.to_string())
    .bind(status)
    .execute(music)
    .await
    .expect("seed catalog piece");
    piece.to_string()
}

/// Queued `discord_notify` payloads whose `field` is `id`.
async fn queued(worker: &PgPool, field: &str, id: &str) -> Vec<serde_json::Value> {
    sqlx::query_scalar(
        "SELECT payload_json::jsonb FROM jobs.mq_payloads \
         WHERE name = 'discord_notify' AND payload_json->>$1 = $2",
    )
    .bind(field)
    .bind(id)
    .fetch_all(worker)
    .await
    .unwrap()
}

fn best(user: &str, piece: &str, subscore: f32) -> GlobalSeasonBest {
    GlobalSeasonBest {
        user_id: user.to_string(),
        season_id: "2026-09".into(),
        catalog_score_id: piece.to_string(),
        mode: Mode::Tempo,
        subscore,
        achieved_at_ms: 1_790_631_000_000,
    }
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn only_beating_another_players_record_is_announced() {
    let (worker, music) = pools().await;
    let piece = seed_piece(&music, "accepted").await;
    let (a, b) = (
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    );
    let repo = PgGlobalLeaderboardRepo::new(music.clone());
    let announced = || queued(&worker, "catalog_score_id", &piece);

    // The first result on a piece is not a record.
    repo.upsert_season_best(&best(&a, &piece, 90.0))
        .await
        .unwrap();
    assert!(announced().await.is_empty());

    // Another player beats it: one anonymous announcement.
    repo.upsert_season_best(&best(&b, &piece, 95.5))
        .await
        .unwrap();
    let jobs = announced().await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["kind"], "season_record");
    assert_eq!(jobs[0]["mode"], "tempo");
    assert_eq!(jobs[0]["achieved_on"], "2026-09-28");
    assert!(
        !jobs[0].to_string().contains(&b),
        "the player never reaches the payload"
    );

    // The holder improving their own record, or a lower result: nothing more.
    repo.upsert_season_best(&best(&b, &piece, 97.0))
        .await
        .unwrap();
    repo.upsert_season_best(&best(&a, &piece, 92.0))
        .await
        .unwrap();
    assert_eq!(announced().await.len(), 1);

    // And the bests themselves are stored as before.
    let rows = repo.season_bests("2026-09", Mode::Tempo).await.unwrap();
    assert_eq!(
        rows.iter().filter(|r| r.catalog_score_id == piece).count(),
        2
    );
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn accepting_a_score_announces_it_and_rejecting_does_not() {
    let (worker, music) = pools().await;
    let repo = PgCatalogSearchRepo::new(music.clone());
    let reviewer = uuid::Uuid::new_v4().to_string();

    let rejected = seed_piece(&music, "pending").await;
    assert!(
        repo.set_moderation_status(&rejected, "rejected", &reviewer, Some("no"))
            .await
            .unwrap()
    );
    assert!(
        queued(&worker, "catalog_score_id", &rejected)
            .await
            .is_empty()
    );

    let accepted = seed_piece(&music, "pending").await;
    assert!(
        repo.set_moderation_status(&accepted, "accepted", &reviewer, None)
            .await
            .unwrap()
    );
    let jobs = queued(&worker, "catalog_score_id", &accepted).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["kind"], "score_accepted");
    // It waits for the grouping window, so a burst of acceptances shares a message.
    let waits: bool = sqlx::query_scalar(
        "SELECT m.attempt_at > now() + interval '9 minutes' \
         FROM jobs.mq_msgs m JOIN jobs.mq_payloads p ON p.id = m.id \
         WHERE p.name = 'discord_notify' AND p.payload_json->>'catalog_score_id' = $1",
    )
    .bind(&accepted)
    .fetch_one(&worker)
    .await
    .unwrap();
    assert!(
        waits,
        "the catalog announcement is delayed by the grouping window"
    );
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn accepting_a_soundfont_announces_it_and_rejecting_does_not() {
    let (worker, music) = pools().await;
    let repo = PgSoundFontRepo::new(music.clone());
    let reviewer = uuid::Uuid::new_v4().to_string();
    let mut ids = Vec::new();
    for _ in 0..2 {
        let id = format!("test-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO music.soundfonts (id, label, object_key, license) \
             VALUES ($1, 'Test font', $1, 'CC0-1.0')",
        )
        .bind(&id)
        .execute(&music)
        .await
        .expect("seed soundfont");
        ids.push(id);
    }

    assert!(
        repo.set_moderation_status(&ids[0], "rejected", &reviewer, Some("no"))
            .await
            .unwrap()
    );
    assert!(queued(&worker, "soundfont_id", &ids[0]).await.is_empty());

    assert!(
        repo.set_moderation_status(&ids[1], "accepted", &reviewer, None)
            .await
            .unwrap()
    );
    let jobs = queued(&worker, "soundfont_id", &ids[1]).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["kind"], "sound_font_accepted");

    // An unknown id updates nothing and announces nothing.
    assert!(
        !repo
            .set_moderation_status("test-missing", "accepted", &reviewer, None)
            .await
            .unwrap()
    );
    assert!(
        queued(&worker, "soundfont_id", "test-missing")
            .await
            .is_empty()
    );
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn the_worker_reads_only_accepted_subjects() {
    use cymbra_discord::AnnouncementSource;
    use cymbra_discord::pg::PgAnnouncementSource;

    let (_worker, music) = pools().await;
    // The worker reads through admin_svc, the cross-schema reader.
    let source = PgAnnouncementSource::new(pool("CYMBRA_ADMIN_DATABASE_URL").await);

    let accepted = seed_piece(&music, "accepted").await;
    let pending = seed_piece(&music, "pending").await;
    let card = source.accepted_score(&accepted).await.unwrap().unwrap();
    assert_eq!(card.title.as_deref(), Some("Test piece"));
    assert_eq!(card.composer, None);
    assert_eq!(source.accepted_score(&pending).await.unwrap(), None);
    assert_eq!(source.accepted_score("not-a-uuid").await.unwrap(), None);

    let font = format!("test-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO music.soundfonts (id, label, object_key, license, attribution, moderation_status) \
         VALUES ($1, 'Test font', $1, 'CC-BY-4.0', 'Someone', 'accepted')",
    )
    .bind(&font)
    .execute(&music)
    .await
    .unwrap();
    // The grouped catalog message's candidates: accepted items only.
    let recent = source.recently_accepted(24).await.unwrap();
    let keys: Vec<String> = recent.iter().map(|i| i.dedup_key()).collect();
    assert!(keys.contains(&format!("discord:music.score_accepted:{accepted}")));
    assert!(!keys.contains(&format!("discord:music.score_accepted:{pending}")));
    let font_item = recent
        .iter()
        .find(|i| i.dedup_key() == format!("discord:music.soundfont_accepted:{font}"))
        .expect("the accepted SoundFont is a candidate");
    match font_item {
        cymbra_discord::CatalogItem::SoundFont { card, .. } => {
            assert_eq!(card.label, "Test font");
            assert_eq!(card.instrument, "keyboard");
            assert_eq!(card.attribution.as_deref(), Some("Someone"));
        }
        other => panic!("not a SoundFont: {other:?}"),
    }
}
