// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! « Traduction en ligne » (change: add-lingua-online-translation): relay one sentence to a
//! translation provider, the next one when it fails, and answer unavailable when none does.
//!
//! Every boundary is a port declared here and implemented by the composition root or a thin
//! adapter: the providers ([`TranslationProvider`]), who may use it and how much
//! ([`OnlineTranslationPolicy`] — runtime flags), the per-account day ([`DailyCounter`] — the
//! platform cache) and the usage figures ([`UsageRepo`] — daily totals, no account, no text).
//!
//! Keeping nothing: neither the sentence nor the translation is ever logged, stored, put in a
//! metric or an error. The log line per request holds the account, the pair, the provider, the
//! outcome, the latency and the token counts.

use crate::online_translation_core::{self as core, Checked, Refusal};
use async_trait::async_trait;
use cymbra_platform::cache::Cache;
use cymbra_platform::identity::AuthIdentity;
use cymbra_platform::{AppError, Result, guard};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use mockall::automock;

/// What a provider answered: its text (the structured JSON) and what it was billed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderReply {
    pub text: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

/// One translation provider (a thin HTTPS adapter). Its error never carries the request.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait TranslationProvider: Send + Sync {
    /// Stable name for logs and usage figures (`gemini` | `anthropic`).
    fn name(&self) -> &'static str;
    async fn translate(&self, system: &str, user: &str) -> Result<ProviderReply>;
}

/// The runtime knobs (flags), read on every request.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait OnlineTranslationPolicy: Send + Sync {
    /// Whether this caller has access: the flag evaluated with their Lingua betas. Off
    /// withdraws access from everyone without a release.
    async fn has_access(&self, id: &AuthIdentity) -> Result<bool>;
    /// Answered requests allowed per account per day.
    async fn daily_limit(&self) -> u32;
    /// `(input, output)` price of `provider` in dollars per million tokens.
    async fn prices(&self, provider: &str) -> (f64, f64);
}

/// The per-account count of the day's answered requests.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait DailyCounter: Send + Sync {
    /// Answered requests so far, and when the day's count resets (unix seconds; 0 if none).
    async fn count(&self, account: &str) -> Result<(u32, i64)>;
    /// Count one answered request.
    async fn bump(&self, account: &str) -> Result<()>;
}

/// The outcome a usage total is kept under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageOutcome {
    Answered,
    Failed,
    Limited,
}

impl UsageOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            UsageOutcome::Answered => "answered",
            UsageOutcome::Failed => "failed",
            UsageOutcome::Limited => "limited",
        }
    }
}

/// One request folded into the day's totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageEvent {
    /// Days since the epoch (UTC).
    pub day: i32,
    /// The provider asked, or `none` when the request was refused before any was.
    pub provider: String,
    pub pair: String,
    pub outcome: UsageOutcome,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_micro_usd: i64,
}

/// The daily totals (no account, no text).
#[cfg_attr(test, automock)]
#[async_trait]
pub trait UsageRepo: Send + Sync {
    async fn record(&self, event: &UsageEvent) -> Result<()>;
}

/// What the service answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Translated {
        translation: String,
        marks: Vec<(u32, u32)>,
    },
    Unavailable,
    LimitReached {
        resets_at: i64,
    },
}

/// The request as the gRPC layer hands it over.
#[derive(Debug, Clone)]
pub struct Request {
    pub sentence: String,
    pub selection_start: u32,
    pub selection_end: u32,
    pub source_language: String,
    pub target_language: String,
}

pub struct OnlineTranslationModule {
    providers: Vec<Arc<dyn TranslationProvider>>,
    policy: Arc<dyn OnlineTranslationPolicy>,
    counter: Arc<dyn DailyCounter>,
    usage: Arc<dyn UsageRepo>,
}

fn today() -> i32 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (secs / 86_400) as i32
}

impl OnlineTranslationModule {
    /// `providers` in the order they are asked: the primary first.
    pub fn new(
        providers: Vec<Arc<dyn TranslationProvider>>,
        policy: Arc<dyn OnlineTranslationPolicy>,
        counter: Arc<dyn DailyCounter>,
        usage: Arc<dyn UsageRepo>,
    ) -> Self {
        Self {
            providers,
            policy,
            counter,
            usage,
        }
    }

    /// A usage total that could not be written costs the reader nothing: logged, not raised.
    async fn record(&self, event: UsageEvent) {
        if let Err(e) = self.usage.record(&event).await {
            tracing::warn!(error = %e, provider = %event.provider, "online translation usage not recorded");
        }
    }

    pub async fn translate(&self, id: &AuthIdentity, req: &Request) -> Result<Outcome> {
        let checked = core::check(
            &req.sentence,
            req.selection_start,
            req.selection_end,
            &req.source_language,
            &req.target_language,
        )
        .map_err(|r| {
            AppError::InvalidArgument(
                match r {
                    Refusal::Empty => "the sentence is empty",
                    Refusal::TooLong => "the sentence is longer than 1000 characters",
                    Refusal::BadSelection => "the selection is not a passage of the sentence",
                    Refusal::BadPair => "the pair is not two different Lingua languages",
                }
                .into(),
            )
        })?;
        let pair = checked.pair();

        // Access is withdrawn from everyone with the flag off: no provider is asked.
        if !self.policy.has_access(id).await? {
            tracing::info!(account = %id.user_id, %pair, outcome = "no_access", "online translation");
            return Ok(Outcome::Unavailable);
        }

        // Lingua staff are not limited, and still counted in the figures.
        let staff = guard::is_staff_in_scope(id, cymbra_platform::LINGUA_SCOPE);
        if !staff {
            let (used, resets_at) = self.counter.count(&id.user_id).await?;
            if used >= self.policy.daily_limit().await {
                tracing::info!(account = %id.user_id, %pair, outcome = "limited", "online translation");
                self.record(UsageEvent {
                    day: today(),
                    provider: "none".into(),
                    pair,
                    outcome: UsageOutcome::Limited,
                    input_tokens: 0,
                    output_tokens: 0,
                    cost_micro_usd: 0,
                })
                .await;
                return Ok(Outcome::LimitReached { resets_at });
            }
        }

        let user = core::user_prompt(&checked);
        for provider in &self.providers {
            if let Some(outcome) = self
                .ask(provider.as_ref(), &checked, &user, id, staff)
                .await
            {
                return Ok(outcome);
            }
        }
        Ok(Outcome::Unavailable)
    }

    /// One provider: `Some` when it answered something readable, `None` to move on.
    async fn ask(
        &self,
        provider: &dyn TranslationProvider,
        checked: &Checked,
        user: &str,
        id: &AuthIdentity,
        staff: bool,
    ) -> Option<Outcome> {
        let name = provider.name();
        let pair = checked.pair();
        let started = Instant::now();
        let reply = provider.translate(core::SYSTEM_PROMPT, user).await;
        let latency_ms = started.elapsed().as_millis() as u64;
        let (input_price, output_price) = self.policy.prices(name).await;
        let (reply, answer) = match reply {
            Ok(r) => {
                let answer = core::read_answer(&r.text, checked);
                (r, answer)
            }
            Err(e) => {
                tracing::warn!(account = %id.user_id, %pair, provider = name, latency_ms, error = %e, outcome = "failed", "online translation");
                self.record(UsageEvent {
                    day: today(),
                    provider: name.into(),
                    pair,
                    outcome: UsageOutcome::Failed,
                    input_tokens: 0,
                    output_tokens: 0,
                    cost_micro_usd: 0,
                })
                .await;
                return None;
            }
        };
        let cost = core::cost_micro_usd(
            reply.input_tokens,
            reply.output_tokens,
            input_price,
            output_price,
        );
        let Some(answer) = answer else {
            // Billed but unreadable: counted as a failure, at its cost.
            tracing::warn!(account = %id.user_id, %pair, provider = name, latency_ms, input_tokens = reply.input_tokens, output_tokens = reply.output_tokens, outcome = "unreadable", "online translation");
            self.record(UsageEvent {
                day: today(),
                provider: name.into(),
                pair,
                outcome: UsageOutcome::Failed,
                input_tokens: reply.input_tokens,
                output_tokens: reply.output_tokens,
                cost_micro_usd: cost,
            })
            .await;
            return None;
        };
        if !staff && let Err(e) = self.counter.bump(&id.user_id).await {
            tracing::warn!(error = %e, "online translation day count not bumped");
        }
        tracing::info!(account = %id.user_id, %pair, provider = name, latency_ms, input_tokens = reply.input_tokens, output_tokens = reply.output_tokens, outcome = "answered", "online translation");
        self.record(UsageEvent {
            day: today(),
            provider: name.into(),
            pair,
            outcome: UsageOutcome::Answered,
            input_tokens: reply.input_tokens,
            output_tokens: reply.output_tokens,
            cost_micro_usd: cost,
        })
        .await;
        let marks = core::place_marks(&answer.translation, &answer.highlighted);
        Some(Outcome::Translated {
            translation: answer.translation,
            marks,
        })
    }
}

/// The day's count over the platform cache: a counter that expires a day after the first
/// counted request, and the time of that request, to say when it resets.
pub struct CacheDailyCounter {
    cache: Arc<dyn Cache>,
}

const DAY: Duration = Duration::from_secs(86_400);

impl CacheDailyCounter {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    fn keys(account: &str) -> (String, String) {
        (
            format!("lingua_online_translation:count:{account}"),
            format!("lingua_online_translation:since:{account}"),
        )
    }
}

#[async_trait]
impl DailyCounter for CacheDailyCounter {
    async fn count(&self, account: &str) -> Result<(u32, i64)> {
        let (count, since) = Self::keys(account);
        let n = self
            .cache
            .get(&count)
            .await?
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let resets_at = self
            .cache
            .get(&since)
            .await?
            .and_then(|v| v.parse::<i64>().ok())
            .map(|s| s + DAY.as_secs() as i64)
            .unwrap_or(0);
        Ok((n, resets_at))
    }

    async fn bump(&self, account: &str) -> Result<()> {
        let (count, since) = Self::keys(account);
        if self.cache.incr_with_ttl(&count, DAY).await? == 1 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            self.cache.set_ex(&since, &now.to_string(), DAY).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_platform::cache::FakeCache;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    const SENTENCE: &str = "He had been putting off the letter.";

    fn req() -> Request {
        let start = SENTENCE.find("putting off").unwrap() as u32;
        Request {
            sentence: SENTENCE.into(),
            selection_start: start,
            selection_end: start + "putting off".len() as u32,
            source_language: "en".into(),
            target_language: "fr".into(),
        }
    }

    fn reader() -> AuthIdentity {
        AuthIdentity {
            user_id: "u-reader".into(),
            audience: "lingua".into(),
            roles: vec!["user".into()],
            roles_by_scope: BTreeMap::new(),
        }
    }

    fn lingua_admin() -> AuthIdentity {
        AuthIdentity {
            user_id: "u-admin".into(),
            audience: "lingua".into(),
            roles: vec!["admin".into()],
            roles_by_scope: [("lingua".to_string(), vec!["admin".to_string()])].into(),
        }
    }

    const GOOD: &str = r#"{"translation": "Il remettait la lettre à plus tard.", "highlighted": ["remettait", "à plus tard"]}"#;

    fn provider(name: &'static str, reply: Result<&'static str>) -> Arc<dyn TranslationProvider> {
        let mut p = MockTranslationProvider::new();
        p.expect_name().return_const(name);
        let reply = reply.map(|t| ProviderReply {
            text: t.into(),
            input_tokens: 600,
            output_tokens: 60,
        });
        p.expect_translate().returning(move |_, _| match &reply {
            Ok(r) => Ok(r.clone()),
            Err(_) => Err(AppError::Unavailable("provider down".into())),
        });
        Arc::new(p)
    }

    fn untouched(name: &'static str) -> Arc<dyn TranslationProvider> {
        let mut p = MockTranslationProvider::new();
        p.expect_name().return_const(name);
        p.expect_translate().times(0);
        Arc::new(p)
    }

    fn policy(access: bool, limit: u32) -> Arc<dyn OnlineTranslationPolicy> {
        let mut p = MockOnlineTranslationPolicy::new();
        p.expect_has_access().returning(move |_| Ok(access));
        p.expect_daily_limit().return_const(limit);
        p.expect_prices().returning(|_| (0.25, 1.50));
        Arc::new(p)
    }

    #[derive(Default)]
    struct Recorded(Mutex<Vec<UsageEvent>>);
    #[async_trait]
    impl UsageRepo for Recorded {
        async fn record(&self, event: &UsageEvent) -> Result<()> {
            self.0.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    fn module(
        providers: Vec<Arc<dyn TranslationProvider>>,
        policy: Arc<dyn OnlineTranslationPolicy>,
        cache: Arc<FakeCache>,
        usage: Arc<Recorded>,
    ) -> OnlineTranslationModule {
        OnlineTranslationModule::new(
            providers,
            policy,
            Arc::new(CacheDailyCounter::new(cache)),
            usage,
        )
    }

    #[tokio::test]
    async fn the_primary_answers_with_marks_and_is_counted() {
        let usage = Arc::new(Recorded::default());
        let cache = Arc::new(FakeCache::default());
        let m = module(
            vec![provider("gemini", Ok(GOOD)), untouched("anthropic")],
            policy(true, 100),
            cache.clone(),
            usage.clone(),
        );
        let out = m.translate(&reader(), &req()).await.unwrap();
        let t = "Il remettait la lettre à plus tard.";
        let at = |w: &str| {
            let b = t.find(w).unwrap();
            let a = t[..b].encode_utf16().count() as u32;
            (a, a + w.encode_utf16().count() as u32)
        };
        assert_eq!(
            out,
            Outcome::Translated {
                translation: t.into(),
                marks: vec![at("remettait"), at("à plus tard")]
            }
        );
        let events = usage.0.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].outcome, UsageOutcome::Answered);
        assert_eq!(events[0].provider, "gemini");
        assert_eq!(events[0].pair, "en-fr");
        assert_eq!(events[0].cost_micro_usd, 240); // 600 × 0.25 + 60 × 1.50
        assert_eq!(
            CacheDailyCounter::new(cache)
                .count("u-reader")
                .await
                .unwrap()
                .0,
            1
        );
    }

    #[tokio::test]
    async fn a_failing_or_unreadable_primary_falls_back_to_the_secondary() {
        for primary in [
            provider("gemini", Err(AppError::Unavailable("x".into()))),
            provider("gemini", Ok("Wait, I need to apply the tags")),
        ] {
            let usage = Arc::new(Recorded::default());
            let m = module(
                vec![primary, provider("anthropic", Ok(GOOD))],
                policy(true, 100),
                Arc::new(FakeCache::default()),
                usage.clone(),
            );
            assert!(matches!(
                m.translate(&reader(), &req()).await.unwrap(),
                Outcome::Translated { .. }
            ));
            let outcomes: Vec<_> = usage
                .0
                .lock()
                .unwrap()
                .iter()
                .map(|e| (e.provider.clone(), e.outcome))
                .collect();
            assert_eq!(
                outcomes,
                vec![
                    ("gemini".into(), UsageOutcome::Failed),
                    ("anthropic".into(), UsageOutcome::Answered)
                ]
            );
        }
    }

    #[tokio::test]
    async fn both_failing_is_unavailable_and_not_counted() {
        let cache = Arc::new(FakeCache::default());
        let m = module(
            vec![
                provider("gemini", Err(AppError::Unavailable("x".into()))),
                provider("anthropic", Err(AppError::Unavailable("y".into()))),
            ],
            policy(true, 100),
            cache.clone(),
            Arc::new(Recorded::default()),
        );
        assert_eq!(
            m.translate(&reader(), &req()).await.unwrap(),
            Outcome::Unavailable
        );
        assert_eq!(
            CacheDailyCounter::new(cache)
                .count("u-reader")
                .await
                .unwrap()
                .0,
            0
        );
    }

    #[tokio::test]
    async fn without_access_no_provider_is_asked() {
        let m = module(
            vec![untouched("gemini"), untouched("anthropic")],
            policy(false, 100),
            Arc::new(FakeCache::default()),
            Arc::new(Recorded::default()),
        );
        assert_eq!(
            m.translate(&reader(), &req()).await.unwrap(),
            Outcome::Unavailable
        );
    }

    #[tokio::test]
    async fn the_daily_limit_refuses_before_any_provider_and_says_when_it_resets() {
        let cache = Arc::new(FakeCache::default());
        let counter = CacheDailyCounter::new(cache.clone());
        counter.bump("u-reader").await.unwrap();
        counter.bump("u-reader").await.unwrap();
        let usage = Arc::new(Recorded::default());
        let m = module(
            vec![untouched("gemini")],
            policy(true, 2),
            cache,
            usage.clone(),
        );
        let out = m.translate(&reader(), &req()).await.unwrap();
        let Outcome::LimitReached { resets_at } = out else {
            panic!("{out:?}")
        };
        assert!(resets_at > 0);
        assert_eq!(usage.0.lock().unwrap()[0].outcome, UsageOutcome::Limited);
    }

    #[tokio::test]
    async fn lingua_staff_are_not_limited_and_still_counted_in_the_figures() {
        let cache = Arc::new(FakeCache::default());
        let usage = Arc::new(Recorded::default());
        let m = module(
            vec![provider("gemini", Ok(GOOD))],
            policy(true, 0),
            cache.clone(),
            usage.clone(),
        );
        assert!(matches!(
            m.translate(&lingua_admin(), &req()).await.unwrap(),
            Outcome::Translated { .. }
        ));
        assert_eq!(usage.0.lock().unwrap()[0].outcome, UsageOutcome::Answered);
        assert_eq!(
            CacheDailyCounter::new(cache)
                .count("u-admin")
                .await
                .unwrap()
                .0,
            0
        );
    }

    #[tokio::test]
    async fn a_bad_request_is_refused_before_access_is_even_read() {
        let mut p = MockOnlineTranslationPolicy::new();
        p.expect_has_access().times(0);
        let m = OnlineTranslationModule::new(
            vec![untouched("gemini")],
            Arc::new(p),
            Arc::new(CacheDailyCounter::new(Arc::new(FakeCache::default()))),
            Arc::new(Recorded::default()),
        );
        let mut long = req();
        long.sentence = "a".repeat(1_001);
        assert!(matches!(
            m.translate(&reader(), &long).await,
            Err(AppError::InvalidArgument(_))
        ));
        let mut pair = req();
        pair.target_language = "en".into();
        assert!(matches!(
            m.translate(&reader(), &pair).await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    /// The request and the answer never reach a log line, whatever the path: a capture of
    /// everything traced across an answered request and a provider failure holds neither.
    #[tokio::test]
    async fn nothing_of_the_sentence_or_the_translation_is_logged() {
        use std::io::Write;
        #[derive(Clone, Default)]
        struct Sink(Arc<Mutex<Vec<u8>>>);
        impl Write for Sink {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let sink = Sink::default();
        let writer = sink.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(move || writer.clone())
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        struct FailingUsage;
        #[async_trait]
        impl UsageRepo for FailingUsage {
            async fn record(&self, _: &UsageEvent) -> Result<()> {
                Err(AppError::Unavailable("db down".into()))
            }
        }
        let m = OnlineTranslationModule::new(
            vec![
                provider("gemini", Err(AppError::Unavailable("x".into()))),
                provider("anthropic", Ok(GOOD)),
            ],
            policy(true, 100),
            Arc::new(CacheDailyCounter::new(Arc::new(FakeCache::default()))),
            Arc::new(FailingUsage),
        );
        m.translate(&reader(), &req()).await.unwrap();
        let log = String::from_utf8(sink.0.lock().unwrap().clone()).unwrap();
        assert!(
            log.contains("online translation"),
            "the request is logged: {log}"
        );
        for words in ["putting off", "the letter", "remettait", "plus tard"] {
            assert!(!log.contains(words), "page text reached the log: `{words}`");
        }
    }
}
