//! The Discord report queries against Postgres (change: add-discord-notifications,
//! tasks 3.7 and 3.9b): `PgReportSource` over the worker's `admin_svc` pool.
//! Requires the dev infra up (`backend/docker-compose.yml`) with the roles
//! bootstrapped; the test migrates `user_account` and `music` itself and seeds a
//! day of its own, years ahead, so it never reads another test's rows.
//!
//! Run: `cargo test -p cymbra-worker --test discord_reports_it -- --ignored`

use chrono::{Duration, NaiveDate};
use cymbra_discord::pg::PgReportSource;
use cymbra_discord::ports::ReportSource;
use cymbra_discord::reports::{Cadence, Figure, Period};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

async fn pool(var: &str) -> PgPool {
    let url = std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"));
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap_or_else(|e| panic!("connect {var}: {e}"))
}

async fn exec(admin: &PgPool, sql: &str, binds: &[&dyn ToSql]) {
    let mut q = sqlx::query(sql);
    for b in binds {
        q = b.bind_to(q);
    }
    q.execute(admin)
        .await
        .unwrap_or_else(|e| panic!("{sql}: {e}"));
}

/// A tiny helper so seeds can pass mixed values.
trait ToSql: Sync {
    fn bind_to<'q>(
        &'q self,
        q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>;
}
macro_rules! to_sql {
    ($($t:ty),*) => {$(
        impl ToSql for $t {
            fn bind_to<'q>(
                &'q self,
                q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
            ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
                q.bind(self)
            }
        }
    )*};
}
to_sql!(Uuid, String, &str, f32, chrono::DateTime<chrono::Utc>);

#[tokio::test]
#[ignore = "needs docker compose (Postgres) with per-module roles"]
async fn the_report_queries_count_activity_and_keep_private_scores_out() {
    let user = pool("CYMBRA_USER_DATABASE_URL").await;
    cymbra_user::MIGRATOR
        .run(&user)
        .await
        .expect("migrate user");
    let music = pool("CYMBRA_MUSIC_DATABASE_URL").await;
    cymbra_music::MIGRATOR
        .run(&music)
        .await
        .expect("migrate music");
    let admin = pool("CYMBRA_ADMIN_DATABASE_URL").await;

    // A day of our own, years ahead.
    let day = NaiveDate::from_ymd_opt(2031, 1, 1).unwrap()
        + Duration::days((Uuid::new_v4().as_u128() % 3_000) as i64);
    let at = |h: u32| day.and_hms_opt(h, 0, 0).unwrap().and_utc();
    let period = Period {
        start: day,
        end: day + Duration::days(1),
        cadence: Cadence::Daily,
    };

    // Two accepted catalog pieces (one only settles a consensus) and a SoundFont.
    let (air, other) = (Uuid::new_v4(), Uuid::new_v4());
    for (id, title) in [(air, "Air"), (other, "Other")] {
        exec(
            &admin,
            "INSERT INTO music.catalog_scores (id, source, source_url, source_item_id, license, \
                confidence, sha256, origin_format, conversion_status, object_key, work_key, \
                moderation_status, title, composer, reviewed_at) \
             VALUES ($1, 'test', 'https://example.test', $2, 'CC0-1.0', 'verified', $2, 'mxl', \
                'converted', $2, $2, 'accepted', $3, 'Bach', $4)",
            &[&id, &id.to_string(), &title, &at(9)],
        )
        .await;
    }
    let font = format!("test-{}", Uuid::new_v4());
    exec(
        &admin,
        "INSERT INTO music.soundfonts (id, label, object_key, license, moderation_status, reviewed_at) \
         VALUES ($1, 'Test Grand', $1, 'CC0-1.0', 'accepted', $2)",
        &[&font, &at(10)],
    )
    .await;

    // Three players; a session on a PRIVATE score id; one session the day after.
    let (u1, u2, u3) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let private = Uuid::new_v4().to_string();
    for (u, score, pct, h) in [
        (u1, air.to_string(), 80.0_f32, 11),
        (u1, air.to_string(), 90.0, 12),
        (u2, air.to_string(), 100.0, 13),
        (u3, private.clone(), 70.0, 14),
    ] {
        exec(
            &admin,
            "INSERT INTO music.play_sessions (id, user_id, score_id, played_at, overall_sync_pct) \
             VALUES ($1, $2, $3, $4, $5)",
            &[&Uuid::new_v4(), &u, &score, &at(h), &pct],
        )
        .await;
    }
    exec(
        &admin,
        "INSERT INTO music.play_sessions (id, user_id, score_id, played_at, overall_sync_pct) \
         VALUES ($1, $2, $3, $4, 50)",
        &[
            &Uuid::new_v4(),
            &u1,
            &air.to_string(),
            &(at(12) + Duration::days(1)),
        ],
    )
    .await;
    for u in [u1, u2] {
        exec(
            &admin,
            "INSERT INTO music.score_ratings (user_id, catalog_score_id, verdict, updated_at) \
             VALUES ($1, $2, 'like', $3)",
            &[&u, &air, &at(15)],
        )
        .await;
    }
    exec(
        &admin,
        "INSERT INTO music.score_consensus_settlements \
         (catalog_score_id, truth_positive, avg_effective, rater_count, settled_at) \
         VALUES ($1, TRUE, 0.9, 5, $2)",
        &[&other, &at(16)],
    )
    .await;

    // Three new accounts: Google (then an email added later), email, Apple.
    for (u, locale, providers) in [
        (u1, Some("fr"), vec!["google", "local"]),
        (u2, None, vec!["local"]),
        (u3, Some("en-GB"), vec!["apple"]),
    ] {
        exec(
            &admin,
            "INSERT INTO user_account.users (id, created_at, locale) VALUES ($1, $2, $3)",
            &[&u, &at(8), &locale.unwrap_or("").to_string()],
        )
        .await;
        for (i, p) in providers.iter().enumerate() {
            exec(
                &admin,
                "INSERT INTO user_account.user_identities (id, user_id, provider, subject, linked_at) \
                 VALUES ($1, $2, $3, $4, $5)",
                &[&Uuid::new_v4(), &u, p, &Uuid::new_v4().to_string(), &at(8 + i as u32)],
            )
            .await;
        }
    }

    let source = PgReportSource::new(admin);

    let m = source.music(&period).await.expect("music");
    assert_eq!(m.sessions, Figure::new(4, 3));
    assert_eq!(m.accuracy_pct.map(|a| a.round() as i64), Some(85));
    assert_eq!(m.ratings, Figure::new(2, 2));
    assert_eq!(m.consensus, 1);
    assert_eq!(
        m.accepted,
        vec![
            "Air — Bach".to_string(),
            "Other — Bach".into(),
            "Test Grand".into()
        ]
    );
    // Only the accepted catalog piece is ranked — never the private score.
    assert_eq!(m.top.len(), 1);
    assert_eq!(m.top[0].card.title.as_deref(), Some("Air"));
    assert_eq!((m.top[0].plays, m.top[0].players), (3, 2));

    let id = source.id(&period).await.expect("id");
    assert_eq!(id.new_accounts, 3);
    let mut methods = id.methods.clone();
    methods.sort();
    assert_eq!(
        methods,
        vec![
            ("apple".into(), 1),
            ("google".into(), 1),
            ("local".into(), 1)
        ]
    );
    let mut locales = id.locales.clone();
    locales.sort();
    assert_eq!(
        locales,
        vec![("".into(), 1), ("en-GB".into(), 1), ("fr".into(), 1)]
    );
}
