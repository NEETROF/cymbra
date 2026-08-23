//! Cymbra ID — composition root (binary `cymbra-server`).
//!
//! The **only** place modules are wired: connect Postgres (per-module roles) +
//! Redis, run migrations, build the user + auth modules and their adapters,
//! install the internal-token interceptors, and serve gRPC + the Axum JWKS/health
//! surface. Contains no business logic.

use std::net::SocketAddr;
use std::sync::Arc;

use cymbra_analytics::proto::usage_service_server::UsageServiceServer;
use cymbra_auth::{
    AuthConfig, AuthGrpc, AuthModule, PgCredentialRepo, PgSessionStore, RealOidcVerifier,
};
use cymbra_auth::{CredentialRepo, OidcProviderCfg, OidcVerifier, SessionStore};
use cymbra_auth_port::proto::auth_service_server::AuthServiceServer;
use cymbra_feature_flags::proto::flag_service_server::FlagServiceServer;
use cymbra_music::proto::score_service_server::ScoreServiceServer;
use cymbra_music::{
    PgCatalogSearchRepo, PgOfflineSecretRepo, PgScoreRatingRepo, PgUserLibraryRepo,
    PgUserScoreRepo, ScoreGrpc, ScoreModule,
};
use cymbra_notifications::proto::notification_service_server::NotificationServiceServer;
use cymbra_platform::cache::{Cache, RedisCache};
use cymbra_platform::config::Config;
use cymbra_platform::email::{EmailSender, SmtpSender};
use cymbra_platform::interceptor::{AuthInterceptor, OptionalAuthInterceptor};
use cymbra_platform::{db, metrics, telemetry};
use cymbra_storage::{LocalFirstStore, ObjectStorage, S3Params};
use cymbra_user::{PgUserRepo, UserGrpc, UserModule};
use cymbra_user_port::UserPort;
use cymbra_user_port::proto::user_service_server::UserServiceServer;
use http::{HeaderName, HeaderValue};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load `backend/.env` (repo-root run) or `.env` (run from backend/) if present.
    // Real environment variables always win over the file.
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    let cfg = Config::from_env()?;
    let telemetry = telemetry::init(
        "cymbra-server",
        cfg.otlp_enabled,
        cfg.otlp_endpoint.as_deref(),
    )?;
    metrics::install_resource_metrics();
    let red = std::sync::Arc::new(metrics::RedMetrics::new());

    // --- Postgres (per-module roles) + migrations ---
    let auth_pool = db::connect(&cfg.auth_database_url, 5).await?;
    let user_pool = db::connect(&cfg.user_database_url, 5).await?;
    let ready_pool = user_pool.clone(); // for the readiness probe
    let flags_resolver_pool = user_pool.clone(); // platform-admin scope resolver
    // Push registry (change: add-push-notifications) — its tables live in the
    // `user_account` schema, so it shares this pool.
    let notifications_pool = user_pool.clone();
    cymbra_auth::MIGRATOR.run(&auth_pool).await?;
    cymbra_user::MIGRATOR.run(&user_pool).await?;

    // --- Redis: disposable cache (rate-limit + email throttles) only ---
    // Sessions are durable in Postgres (change: durable-sessions-postgres), so a
    // Redis outage no longer signs anyone out and Redis needs no HA/persistence.
    let cache: Arc<dyn Cache> = Arc::new(RedisCache::connect(&cfg.redis_url).await?);

    // --- user module ---
    // DeleteAccount enqueues a `purge_user` job through the jobs.enqueue seam on
    // the user_svc connection (it holds EXECUTE on jobs.enqueue); the worker then
    // performs the complete cross-schema erasure as admin_svc.
    let user_enqueuer: Arc<dyn cymbra_jobs::Enqueuer> =
        Arc::new(cymbra_jobs::PgEnqueuer::new(user_pool.clone()));
    let user_concrete = Arc::new(
        UserModule::new(PgUserRepo::new(user_pool))
            .with_enqueuer(user_enqueuer)
            // Public-profile age gate (change: add-play-activity-profile).
            .with_min_public_sharing_age(cfg.min_public_sharing_age),
    );
    let user_dyn: Arc<dyn UserPort> = user_concrete.clone();

    // --- auth module ---
    let creds: Arc<dyn CredentialRepo> = Arc::new(PgCredentialRepo::new(auth_pool.clone()));
    // Durable session store on the auth pool (auth_svc owns `auth.sessions`).
    let sessions: Arc<dyn SessionStore> =
        Arc::new(PgSessionStore::new(auth_pool, cfg.token.refresh_ttl));
    let providers: Vec<OidcProviderCfg> = cfg
        .oidc_providers
        .iter()
        .map(|p| OidcProviderCfg {
            provider: p.provider.clone(),
            issuer: p.issuer.clone(),
            audiences: p.audiences.clone(),
            jwks_uri: p.jwks_uri.clone(),
        })
        .collect();
    let oidc: Arc<dyn OidcVerifier> = Arc::new(RealOidcVerifier::new(providers));
    let email: Arc<dyn EmailSender> = Arc::new(SmtpSender::new(&cfg.smtp_url, &cfg.smtp_from)?);
    let auth_cfg = AuthConfig::new(
        cfg.token.access_ttl,
        cfg.token.refresh_ttl,
        cfg.allowed_audiences.clone(),
        cfg.password_min_length,
        cfg.signin_max_attempts,
        cfg.signin_lockout,
        cfg.email_max,
        cfg.email_window,
        cfg.verify_ttl,
        cfg.reset_ttl,
        cfg.email_logo_url.clone(),
    );
    let pending: Arc<dyn cymbra_auth::PendingCredentialStore> =
        Arc::new(cymbra_auth::CachePendingStore::new(cache.clone()));
    let auth = Arc::new(AuthModule::new(
        user_dyn.clone(),
        creds,
        cache.clone(),
        pending,
        email,
        oidc,
        sessions,
        &cfg.token.signing_key_pem,
        &cfg.token.kid,
        auth_cfg,
    )?);
    // Shared handle for the browser web-auth HTTP surface (cookie sign-in/refresh/
    // logout); the gRPC AuthService keeps its own handle below.
    let auth_port: Arc<dyn cymbra_auth_port::AuthPort> = auth.clone();

    // The orphan reaper no longer runs in-process here: it is a scheduled job
    // (`orphan_reap`) executed by cymbra-worker (change: add-job-infrastructure).
    // cymbra-worker MUST be deployed for handle-less accounts to be purged.

    // --- interceptors (strict for user; optional for auth's public methods) ---
    let keys = cymbra_server::interceptor_keys(&cfg)?;
    let strict = AuthInterceptor::new(keys.clone(), cfg.allowed_audiences.clone());
    // SoundFont delivery authenticates the same access tokens (Bearer) as gRPC.
    let soundfont_auth = cymbra_server::JwtAuth::new(keys.clone(), cfg.allowed_audiences.clone());
    let optional = OptionalAuthInterceptor::new(keys, cfg.allowed_audiences.clone());

    let user_svc =
        UserServiceServer::with_interceptor(UserGrpc::new(user_concrete), strict.clone());
    let auth_svc = AuthServiceServer::with_interceptor(AuthGrpc::new(auth), optional.clone());

    // --- push notifications (change: add-push-notifications) ---
    // Device-token registry, per-category consent and the per-user timezone all
    // live in `user_account`, so this shares the user_svc pool. Strict auth: every
    // method acts on the authenticated caller's own account. Always mounted — the
    // registry works with no Firebase project configured; only the worker's send
    // path needs credentials.
    let notification_svc = NotificationServiceServer::with_interceptor(
        cymbra_notifications::NotificationGrpc::new(Arc::new(
            cymbra_notifications::PgPushRegistry::new(notifications_pool),
        )),
        strict.clone(),
    );

    // --- feature flags (shared, app-agnostic; change: add-runtime-feature-flags) ---
    // Mounted behind the OPTIONAL interceptor so `GetEffectiveFlags` works with or
    // without a token (pre-account UI respects kill-switches), while the admin
    // methods still require an authenticated admin. Defaults-only when no flags DB.
    let flag_service = cymbra_server::build_flag_service(&cfg, flags_resolver_pool).await?;
    cymbra_server::spawn_flag_refreshers(&cfg, flag_service.clone());

    // --- plans (free/premium ledger, beta campaigns, access codes; change:
    // add-premium-subscription). Wired only when `CYMBRA_PLANS_DATABASE_URL` is set;
    // otherwise every plan-aware seam below stays on its inert default (everyone is
    // `free`, exactly the pre-plan behaviour). Even when wired, `plans.enabled`
    // (default off) keeps it dark. The withdrawal-on-lapse rotator needs the music
    // offline-secret store, so it gets its own small music connection.
    let plan_service: Option<Arc<cymbra_plans::PlanService>> = match cfg
        .plans_database_url
        .as_deref()
    {
        Some(url) => {
            let plans_pool = cymbra_plans::pg::connect(url, 5).await?;
            cymbra_plans::MIGRATOR.run(&plans_pool).await?;
            let rotator: Option<Arc<dyn cymbra_plans::CacheSecretRotator>> =
                match cfg.music_database_url.as_deref() {
                    Some(music_url) => {
                        let pool = db::connect(music_url, 2).await?;
                        Some(Arc::new(cymbra_server::OfflineSecretRotator::new(
                            Arc::new(PgOfflineSecretRepo::new(pool)),
                        )))
                    }
                    None => None,
                };
            Some(Arc::new(cymbra_plans::PlanService::new(
                cymbra_plans::PlanDeps {
                    entitlements: Arc::new(cymbra_plans::pg::PgEntitlementRepo::new(
                        plans_pool.clone(),
                    )),
                    campaigns: Arc::new(cymbra_plans::pg::PgCampaignRepo::new(plans_pool.clone())),
                    memberships: Arc::new(cymbra_plans::pg::PgMembershipRepo::new(
                        plans_pool.clone(),
                    )),
                    codes: Arc::new(cymbra_plans::pg::PgAccessCodeRepo::new(plans_pool.clone())),
                    billing_events: Arc::new(cymbra_plans::pg::PgBillingEventRepo::new(
                        plans_pool.clone(),
                    )),
                    audit: Arc::new(cymbra_plans::pg::PgAuditRepo::new(plans_pool)),
                    config: Arc::new(cymbra_server::FlagPlanConfig::new(flag_service.clone())),
                    clock: Arc::new(cymbra_plans::SystemClock),
                    rotator,
                },
            )))
        }
        None => {
            tracing::info!("plans disabled (CYMBRA_PLANS_DATABASE_URL unset)");
            None
        }
    };
    let plan_source: Option<Arc<dyn cymbra_plans::PlanSource>> = plan_service
        .clone()
        .map(|p| p as Arc<dyn cymbra_plans::PlanSource>);

    let flag_grpc = match &plan_source {
        Some(p) => cymbra_feature_flags::grpc::FlagGrpc::new(flag_service.clone())
            .with_plans(Arc::new(cymbra_server::PlanContext::new(p.clone()))),
        None => cymbra_feature_flags::grpc::FlagGrpc::new(flag_service.clone()),
    };
    let flag_svc = FlagServiceServer::with_interceptor(flag_grpc, optional);

    // Purchase channels (store aggregator / web MoR), each wired only when its
    // environment block is present; the paywall flags gate them at runtime.
    let billing_channels = cymbra_server::billing::BillingChannels::build(
        &cymbra_server::billing::BillingEnv::from_env(),
    );
    let paywall_cfg: Arc<dyn cymbra_plans::PaywallConfigSource> =
        Arc::new(cymbra_server::FlagPaywallConfig::new(flag_service.clone()));

    // PlanService gRPC (app + music-admin console), strict auth. Unconfigured
    // channels answer `unimplemented`.
    let plan_svc = plan_service.clone().map(|ps| {
        let mut g = cymbra_plans::grpc::PlanGrpc::new(ps, paywall_cfg.clone(), cache.clone())
            .with_handles(Arc::new(cymbra_server::UserPortHandles::new(
                user_dyn.clone(),
            )));
        if let (Some(customers), Some(rc)) =
            (&billing_channels.rc_customers, &billing_channels.revenuecat)
        {
            g = g.with_store(customers.clone(), rc.allow_sandbox);
        }
        if let Some(w) = &billing_channels.web {
            g = g.with_web(w.clone());
        }
        cymbra_plans::proto::plan_service_server::PlanServiceServer::with_interceptor(
            g,
            strict.clone(),
        )
    });
    // Provider notification routes (store aggregator webhook, web webhooks) —
    // mounted only when the plans module is wired.
    let billing_http = plan_service.clone().map(|ps| {
        cymbra_server::billing::billing_router(cymbra_server::billing::BillingState {
            svc: ps,
            paywall: paywall_cfg.clone(),
            channels: billing_channels.clone(),
        })
    });

    // --- analytics UsageService (feature-usage telemetry ingestion; change:
    // add-feature-usage-analytics, design D5/D10). Wired only when BOTH the
    // dedicated `analytics` DB URL AND the bucket master secret are set — a missing
    // secret must never silently fall back to a guessable key. Own pool + one
    // MIGRATOR run (creates analytics.usage_events + the two daily aggregates).
    // Runs behind the strict auth interceptor (ingestion is authenticated, the
    // anti-spam guarantee, design D9). Absent config ⇒ inert.
    let usage_svc = match (
        cfg.analytics_database_url.as_deref(),
        cfg.analytics_bucket_secret.as_deref(),
    ) {
        (Some(db_url), Some(secret)) => {
            let analytics_pool = cymbra_analytics::connect(db_url, 5).await?;
            cymbra_analytics::MIGRATOR.run(&analytics_pool).await?;
            let repo: Arc<dyn cymbra_analytics::UsageEventRepo> = Arc::new(
                cymbra_analytics::PgUsageEventRepo::new(analytics_pool.clone()),
            );
            // The reporting reads (back-office console) share the analytics_svc pool.
            let read: Arc<dyn cymbra_analytics::UsageReadRepo> =
                Arc::new(cymbra_analytics::PgUsageReadRepo::new(analytics_pool));
            let usage = cymbra_analytics::UsageGrpc::new(repo, read, secret.as_bytes().to_vec());
            Some(UsageServiceServer::with_interceptor(usage, strict.clone()))
        }
        _ => {
            tracing::info!(
                "analytics UsageService disabled (CYMBRA_ANALYTICS_DATABASE_URL / \
                 CYMBRA_ANALYTICS_BUCKET_SECRET unset)"
            );
            None
        }
    };

    // SoundFont delivery (change: add-soundfont-delivery): a dedicated PRIVATE store,
    // separate from scores. Built here (before the music module) so it is shared by
    // the ScoreService admin RPCs (delete removes the object) and the delivery/upload
    // route. Unconfigured (bucket unset) ⇒ the routes report unavailable.
    let soundfont_store: Option<Arc<dyn ObjectStorage>> = match cfg.soundfont_storage.as_ref() {
        Some(sf) => Some(Arc::new(LocalFirstStore::from_config(
            &sf.local_root,
            &S3Params {
                bucket: sf.bucket.clone(),
                endpoint: sf.endpoint.clone(),
                region: sf.region.clone(),
                access_key: sf.access_key.clone(),
                secret_key: sf.secret_key.clone(),
                allow_http: sf.allow_http,
            },
        )?)),
        None => {
            tracing::info!("soundfont delivery disabled (CYMBRA_SOUNDFONT_S3_BUCKET unset)");
            None
        }
    };

    // --- music module (owns the `music` schema via `music_svc`) ---
    // Wired whenever a music DB is configured. Own pool + one MIGRATOR run, then:
    //   * PlayService — reliable end-of-session play stats + the profile heatmap
    //     (change: add-play-activity-profile). Needs NO object store, so it is
    //     available on the music DB alone. Its cross-schema-free visibility gate is
    //     the injected user port (`user_dyn`).
    //   * ScoreService — user uploads, ADDITIONALLY when the object store is set.
    // Both run behind the strict auth interceptor. Absent music DB ⇒ both inert.
    let (
        play_svc,
        score_svc,
        soundfont_repo,
        user_soundfont_repo,
        leaderboard_svc,
        global_leaderboard_svc,
        score_preview_parts,
    ) = match cfg.music_database_url.as_deref() {
        Some(db_url) => {
            let music_pool = db::connect(db_url, 5).await?;
            cymbra_music::MIGRATOR.run(&music_pool).await?;
            // Persisted SoundFont catalog (change: add-soundfont-catalog-db): one
            // source of truth read by both the ListSoundFonts RPC and the delivery
            // route below.
            let soundfont_repo: Arc<dyn cymbra_music::SoundFontRepo> =
                Arc::new(cymbra_music::PgSoundFontRepo::new(music_pool.clone()));
            // Private, per-user SoundFont library (change: add-soundfont-moderation):
            // owner-scoped store behind the `/me/soundfonts` routes.
            let user_soundfont_repo: Arc<dyn cymbra_music::UserSoundFontRepo> =
                Arc::new(cymbra_music::PgUserSoundFontRepo::new(music_pool.clone()));
            // Shared play-session port: feeds both the PlayService and the catalog
            // access limiter's play-aware download allowance (change: add-catalog-
            // access-limits).
            let play_repo: Arc<dyn cymbra_music::PlayRepo> =
                Arc::new(cymbra_music::PgPlayRepo::new(music_pool.clone()));
            // Curation rewards (change: add-curation-rewards): one module backs the
            // score module's producer seam (record engagement + award coverage on
            // rating, settle honesty on a moderator decision), the reward RPCs on the
            // score service (profile / shop / redeem / reliability), AND the play
            // ingest's engagement signal (change: add-post-play-rating-prompt), so it
            // is built here — above the PlayService — rather than inside the
            // storage-gated score block below.
            let curation_repo: Arc<dyn cymbra_music::CurationRewardsRepo> =
                Arc::new(cymbra_music::PgCurationRewardsRepo::new(music_pool.clone()));
            let rewards_module = {
                let m = cymbra_music::CurationRewardsModule::new(curation_repo.clone());
                // Premium owns the whole shop (change: add-premium-subscription).
                Arc::new(match &plan_source {
                    Some(p) => m.with_plans(p.clone()),
                    None => m,
                })
            };
            // Practice streak (change: add-practice-streak): advanced on the play
            // ingest and read by the app-bar chip. Its freeze cost + grace window
            // are resolved from the flag service on every call, so the back office
            // retunes them without a redeploy. The streak BADGES are not wired from
            // here: the achievement registry derives its own `longest_streak` from
            // the session tables (change: add-achievement-badges).
            let streak_module = Arc::new(
                cymbra_music::StreakModule::new(Arc::new(cymbra_music::PgStreakRepo::new(
                    music_pool.clone(),
                )))
                .with_config_source(Arc::new(
                    cymbra_server::FlagStreakConfig::new(flag_service.clone()),
                )),
            );
            let rewards_sink: Arc<dyn cymbra_music::CurationRewardsSink> = rewards_module.clone();
            // Achievement badges (change: add-achievement-badges): the cross-domain
            // registry read by GetAchievements. It shares the curation repo rather
            // than re-deriving the rating counters a second way, and gathers every
            // other family's counters in the same pass (design D4).
            let badges_module = Arc::new(cymbra_music::BadgesModule::new(Arc::new(
                cymbra_music::PgBadgeRepo::new(music_pool.clone(), curation_repo),
            )));
            // Shared catalog port: the score module reads it, and the play ingest needs
            // it to tell a real catalog score from an upload before recording the
            // engagement signal (change: add-post-play-rating-prompt).
            let catalog_repo: Arc<dyn cymbra_music::CatalogSearchRepo> =
                Arc::new(PgCatalogSearchRepo::new(music_pool.clone()));
            // Leaderboards (change: add-play-leaderboards): the bests store is
            // maintained on the play-session ingest path (as a sink) and read by
            // the LeaderboardService; both share one module + the injected user
            // port for the public/eligible listing gate (cross-schema-free).
            // The GLOBAL, seasonal boards (change: add-global-leaderboard): season
            // bests are maintained by chaining off the per-piece ingest hook (one
            // accepted-catalog + integrity gate for both boards), and read by the
            // GlobalLeaderboardService behind the same user-port listing gate.
            let global_leaderboard_module = Arc::new(cymbra_music::GlobalLeaderboardModule::new(
                Arc::new(cymbra_music::PgGlobalLeaderboardRepo::new(
                    music_pool.clone(),
                )),
                user_dyn.clone(),
            ));
            let global_sink: Arc<dyn cymbra_music::GlobalSeasonSink> =
                global_leaderboard_module.clone();
            let leaderboard_module = Arc::new(
                cymbra_music::LeaderboardModule::new(
                    Arc::new(cymbra_music::PgLeaderboardRepo::new(music_pool.clone())),
                    user_dyn.clone(),
                )
                .with_global(global_sink),
            );
            let leaderboard_sink: Arc<dyn cymbra_music::LeaderboardSink> =
                leaderboard_module.clone();
            let play_module = Arc::new(
                cymbra_music::PlayModule::new(play_repo.clone(), user_dyn.clone())
                    .with_leaderboard(leaderboard_sink),
            );
            let play_svc = Some(
                cymbra_music::proto::play_service_server::PlayServiceServer::with_interceptor(
                    cymbra_music::PlayGrpc::new(play_module)
                        .with_rewards(rewards_sink.clone(), catalog_repo.clone())
                        .with_streak(streak_module.clone()),
                    strict.clone(),
                ),
            );
            let leaderboard_svc = Some(
                    cymbra_music::proto::leaderboard_service_server::LeaderboardServiceServer::with_interceptor(
                        cymbra_music::LeaderboardGrpc::new(leaderboard_module),
                        strict.clone(),
                    ),
                );
            let global_leaderboard_svc = Some(
                    cymbra_music::proto::global_leaderboard_service_server::GlobalLeaderboardServiceServer::with_interceptor(
                        cymbra_music::GlobalLeaderboardGrpc::new(global_leaderboard_module),
                        strict.clone(),
                    ),
                );

            let (score_svc, score_preview_parts) = match cfg.score_storage.as_ref() {
                Some(s3) => {
                    let storage: Arc<dyn ObjectStorage> = Arc::new(LocalFirstStore::from_config(
                        &cfg.score_local_root,
                        &S3Params {
                            bucket: s3.bucket.clone(),
                            endpoint: s3.endpoint.clone(),
                            region: s3.region.clone(),
                            access_key: s3.access_key.clone(),
                            secret_key: s3.secret_key.clone(),
                            allow_http: s3.allow_http,
                        },
                    )?);
                    // Shared rating port: feeds both the score module and the limiter's
                    // engagement allowance (a rating earns download headroom like a play).
                    let rating_repo: Arc<dyn cymbra_music::ScoreRatingRepo> =
                        Arc::new(PgScoreRatingRepo::new(music_pool.clone()));
                    // Interactive course catalog (change: add-notation-courses): the
                    // server-stored manifests read by ListCourses/GetCourse.
                    let course_repo: Arc<dyn cymbra_music::CourseRepo> =
                        Arc::new(cymbra_music::PgCourseRepo::new(music_pool.clone()));
                    // Per-user course completion, cross-device (change: add-notation-courses).
                    let course_progress: Arc<dyn cymbra_music::CourseProgressStore> =
                        Arc::new(cymbra_music::PgCourseProgressStore::new(music_pool.clone()));
                    let module = ScoreModule::new(
                        Arc::new(PgUserScoreRepo::new(music_pool.clone())),
                        catalog_repo.clone(),
                        Arc::new(PgUserLibraryRepo::new(music_pool.clone())),
                        rating_repo.clone(),
                        storage.clone(),
                        cfg.upload_quota_max,
                        cfg.upload_quota_window_days,
                        cfg.upload_max_bytes,
                    )
                    // Resolve proposer attribution (change: add-score-catalog-proposal).
                    .with_user(user_dyn.clone())
                    // Award coverage / settle honesty (change: add-curation-rewards).
                    .with_rewards(rewards_sink)
                    // Persist the per-user offline-cache secret (change:
                    // add-offline-score-cache); the default is an in-memory fake.
                    .with_offline_secrets(Arc::new(PgOfflineSecretRepo::new(music_pool.clone())))
                    // Per-plan upload quota + library cap (change:
                    // add-premium-subscription), flag-backed with the env values
                    // as the free defaults.
                    .with_score_quotas(Arc::new(cymbra_server::FlagScoreQuotas::new(
                        flag_service.clone(),
                        cymbra_music::ScoreQuotas {
                            upload_max: cfg.upload_quota_max,
                            upload_window_days: cfg.upload_quota_window_days,
                            library_max: None,
                        },
                    )))
                    // The freemium daily-access gate on catalog player-opens
                    // (change: add-score-daily-access-rewards): flag-backed
                    // (OFF by default, staff-only rollout first); the
                    // subscription seam is the plan ledger when wired.
                    .with_daily_access(Arc::new(
                        cymbra_music::CatalogDailyAccess::new(Arc::new(
                            cymbra_music::PgCatalogDayAccessRepo::new(music_pool.clone()),
                        ))
                        .with_config_source(Arc::new(cymbra_server::FlagDailyAccessConfig::new(
                            flag_service.clone(),
                        )))
                        .with_subscriptions(match &plan_source {
                            Some(p) => Arc::new(cymbra_music::PlanSubscriptions(p.clone()))
                                as Arc<dyn cymbra_music::SubscriptionSource>,
                            None => Arc::new(cymbra_music::NoSubscriptions),
                        }),
                    ));
                    // Premium gets the extended score quotas (change:
                    // add-premium-subscription).
                    // The shared flag service also powers the drum-audience
                    // gate (change: add-drums-access) — without it the gate
                    // fails closed for everyone, staff included.
                    let module = module.with_flags(flag_service.clone());
                    let module = Arc::new(match &plan_source {
                        Some(p) => module.with_plans(p.clone()),
                        None => module,
                    });
                    // Per-user scrape guardrail over the shared Redis cache + play &
                    // rating ports (engagement = plays + ratings).
                    // Thresholds resolve per request through the flag service (with
                    // the env config as their defaults), so a retune or the
                    // kill-switch applies without a redeploy.
                    let limiter = Arc::new(
                        cymbra_music::CatalogAccessLimiter::new(
                            cache.clone(),
                            play_repo.clone(),
                            rating_repo,
                            cfg.catalog_limits.clone(),
                        )
                        .with_config_source(Arc::new(
                            cymbra_server::FlagCatalogLimitsConfig::new(
                                flag_service.clone(),
                                cfg.catalog_limits.clone(),
                            ),
                        )),
                    );
                    let score_svc = Some(ScoreServiceServer::with_interceptor(
                        ScoreGrpc::new(module.clone())
                            .with_limiter(limiter)
                            .with_soundfonts(soundfont_repo.clone())
                            .with_soundfont_store_opt(soundfont_store.clone())
                            .with_courses(course_repo.clone())
                            .with_course_progress(course_progress.clone())
                            .with_rewards(rewards_module)
                            .with_badges(badges_module)
                            // Soundfont uploader attribution (change:
                            // add-soundfont-uploader-attribution).
                            .with_user_port(user_dyn.clone()),
                        strict.clone(),
                    ));
                    // Score audio teasers (change: add-score-daily-access-rewards):
                    // the HTTP delivery needs the score store + catalog; the inline
                    // regenerate additionally needs the SoundFont store for the
                    // configured preview font (absent ⇒ regenerate reports 503).
                    let renderer = soundfont_store.clone().map(|font_store| {
                        Arc::new(cymbra_music::ScorePreviewRenderer::new(
                            storage.clone(),
                            font_store,
                            catalog_repo.clone(),
                            soundfont_repo.clone(),
                            Arc::new(cymbra_server::FlagScorePreviewConfig::new(
                                flag_service.clone(),
                            )),
                        ))
                    });
                    (
                        score_svc,
                        Some((storage.clone(), catalog_repo.clone(), renderer, module)),
                    )
                }
                None => {
                    tracing::info!("score-upload disabled (CYMBRA_SCORE_S3_BUCKET unset)");
                    (None, None)
                }
            };
            (
                play_svc,
                score_svc,
                Some(soundfont_repo),
                Some(user_soundfont_repo),
                leaderboard_svc,
                global_leaderboard_svc,
                score_preview_parts,
            )
        }
        None => {
            // Fail-fast: S3 configured but no music DB is a misconfiguration.
            if cfg.score_storage.is_some() {
                return Err(anyhow::anyhow!(
                    "CYMBRA_SCORE_S3_BUCKET is set but CYMBRA_MUSIC_DATABASE_URL is missing"
                ));
            }
            tracing::info!("music services disabled (CYMBRA_MUSIC_DATABASE_URL unset)");
            (None, None, None, None, None, None, None)
        }
    };

    // --- HTTP surface (JWKS + health + web-auth cookie endpoints) ---
    // The web-auth surface (change: add-web-auth-cookies) carries the refresh token
    // in an HttpOnly cookie for the browser back office; it reuses the back-office
    // CORS allow-list (credentialed, exact-origin) and the refresh TTL from token cfg.
    let jwks = cymbra_server::jwks_value(&cfg)?;
    let web_auth_cfg = cymbra_server::WebAuthConfig {
        cookie_domain: cfg.web_auth_cookie_domain.clone(),
        cookie_secure: cfg.web_auth_cookie_secure,
        cookie_path: cymbra_server::WebAuthConfig::DEFAULT_PATH.to_string(),
        refresh_ttl: cfg.token.refresh_ttl,
        // Site + back office (change: add-site-account-pages).
        allowed_origins: cfg.web_origins.clone(),
    };
    let soundfont_auth: Arc<dyn cymbra_server::SoundfontAuth> = Arc::new(soundfont_auth);
    let web_plans_auth = soundfont_auth.clone();
    // Score audio-teaser routes (change: add-score-daily-access-rewards); the same
    // auth seam as the SoundFont routes.
    let (sp_store, sp_catalog, sp_renderer, sp_drums) = match score_preview_parts {
        Some((store, catalog, renderer, module)) => {
            // The preview route resolves drum eligibility through the SAME
            // module predicate the RPCs use (change: add-drums-access).
            let drums: Arc<dyn cymbra_music::DrumsEligibility> = module;
            (Some(store), Some(catalog), renderer, Some(drums))
        }
        None => (None, None, None, None),
    };
    let score_preview_state = cymbra_server::ScorePreviewState {
        store: sp_store,
        catalog: sp_catalog,
        renderer: sp_renderer,
        auth: soundfont_auth.clone(),
        drums: sp_drums,
    };
    let soundfont_state = cymbra_server::SoundfontState {
        store: soundfont_store,
        // The persisted catalog (change: add-soundfont-catalog-db) resolves id →
        // object_key; absent music DB ⇒ the route reports 503.
        repo: soundfont_repo,
        // Private per-user library (change: add-soundfont-moderation) backing the
        // `/me/soundfonts` routes.
        user_repo: user_soundfont_repo,
        auth: soundfont_auth,
        // Premium unlocks the whole catalog library and the extended private quota
        // (change: add-premium-subscription).
        plans: plan_source.clone(),
        library_quota: Some(Arc::new(cymbra_server::FlagLibraryQuota::new(
            flag_service.clone(),
        ))),
    };
    let http = cymbra_server::http_router(jwks, ready_pool, cache.clone())
        .merge(cymbra_server::web_auth_router(auth_port, web_auth_cfg))
        .merge(cymbra_server::soundfont_router(
            soundfont_state,
            cfg.back_office_origins.clone(),
        ))
        .merge(cymbra_server::score_preview_router(
            score_preview_state,
            cfg.back_office_origins.clone(),
        ));
    // Provider notification routes (change: add-premium-subscription).
    let http = match billing_http {
        Some(r) => http.merge(r),
        None => http,
    };
    // Browser account summary for the public site (change: add-site-account-pages):
    // handle + linked sign-in methods, read-only, same bearer seam + web origins.
    let http = http.merge(cymbra_server::web_account_router(
        cymbra_server::WebAccountState {
            users: user_dyn.clone(),
            auth: web_plans_auth.clone(),
        },
        cfg.web_origins.clone(),
    ));
    // Browser plan JSON routes for the public site (change: add-site-account-pages):
    // bearer-authenticated through the same seam as the SoundFont routes, CORS
    // restricted to the web origins. Mounted only when the plans module is wired.
    let http = match plan_service.clone() {
        Some(ps) => http.merge(cymbra_server::web_plans_router(
            cymbra_server::WebPlansState {
                svc: ps,
                paywall: paywall_cfg.clone(),
                cache: cache.clone(),
                web: billing_channels
                    .web
                    .clone()
                    .map(|w| w as Arc<dyn cymbra_plans::WebBillingProvider>),
                auth: web_plans_auth,
            },
            cfg.web_origins.clone(),
        )),
        None => http,
    };

    let grpc_addr: SocketAddr = cfg.grpc_addr.parse()?;
    let http_addr: SocketAddr = cfg.http_addr.parse()?;
    tracing::info!(%grpc_addr, %http_addr, "cymbra-server serving");

    // Browser transport for the back office (change: add-moderation-back-office):
    // gRPC-web + a CORS layer restricted to the configured origin(s). gRPC-web is a
    // framing of gRPC (not REST), and the native HTTP/2 gRPC surface the app uses is
    // unchanged — `accept_http1(true)` only *additionally* accepts gRPC-web over
    // HTTP/1.1. Every method still runs behind the same auth interceptor + role
    // guards, so CORS is defence-in-depth, not the authorization boundary. An empty
    // origin list (the default) allows no cross-origin browser access.
    let cors_origins: Vec<HeaderValue> = cfg
        .back_office_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(cors_origins))
        .allow_headers(Any)
        .allow_methods(Any)
        // gRPC-web carries the call status in trailers surfaced as these headers.
        .expose_headers([
            HeaderName::from_static("grpc-status"),
            HeaderName::from_static("grpc-message"),
            HeaderName::from_static("grpc-status-details-bin"),
        ]);

    let mut router = Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(GrpcWebLayer::new())
        .layer(metrics::ObserveLayer::new(red))
        .add_service(user_svc)
        .add_service(auth_svc)
        .add_service(flag_svc)
        .add_service(notification_svc);
    if let Some(score_svc) = score_svc {
        router = router.add_service(score_svc);
    }
    if let Some(play_svc) = play_svc {
        router = router.add_service(play_svc);
    }
    if let Some(leaderboard_svc) = leaderboard_svc {
        router = router.add_service(leaderboard_svc);
    }
    if let Some(global_leaderboard_svc) = global_leaderboard_svc {
        router = router.add_service(global_leaderboard_svc);
    }
    if let Some(usage_svc) = usage_svc {
        router = router.add_service(usage_svc);
    }
    if let Some(plan_svc) = plan_svc {
        router = router.add_service(plan_svc);
    }
    let grpc = router.serve(grpc_addr);
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    let http_srv = axum::serve(listener, http.into_make_service());

    let result = tokio::try_join!(async { grpc.await.map_err(anyhow::Error::from) }, async {
        http_srv.await.map_err(anyhow::Error::from)
    });
    telemetry.shutdown();
    result?;
    Ok(())
}
