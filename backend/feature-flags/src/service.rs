//! The flag/config evaluation + admin service.
//!
//! Holds the code [`Registry`], the override [`FlagStore`] seam (the DB source of
//! truth, or `None` for defaults-only mode), an [`InvalidationBus`], and an
//! [`AdminScopeResolver`]. Reads go through an **L1 in-process snapshot** of the
//! whole override set (near-zero latency); a background refresher keeps it fresh
//! on a TTL backstop and on an invalidation ping (design D2). Evaluation is
//! fail-safe: no store / a failed refresh / an outage all resolve to the code
//! defaults, and kill-switches stay in their safe (disabled) state.

use crate::context::{EvalContext, RolloutScope};
use crate::invalidation::InvalidationBus;
use crate::registry::{KeyDef, Registry};
use crate::resolver::AdminScopeResolver;
use crate::store::{FlagStore, OverrideWrite, StoredOverride};
use crate::value::{FlagValue, ValueType};
use cymbra_platform::error::{AppError, Result};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// The L1 snapshot: the whole override set, keyed by `(app, key)`.
#[derive(Debug, Default)]
struct Snapshot {
    overrides: HashMap<(String, String), StoredOverride>,
    loaded_at: Option<Instant>,
}

/// The resolved state of a declared key for the admin surface.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyState {
    pub def: KeyDef,
    /// The stored override value if any, else the code default.
    pub effective: FlagValue,
    pub has_override: bool,
    /// The override's rollout scope if overridden, else the declared default.
    pub rollout: RolloutScope,
    /// Last editor's account id (`None` when on the code default).
    pub updated_by: Option<String>,
    /// RFC-3339 timestamp of the last override write (`None` when on the default).
    pub updated_at: Option<String>,
}

/// One entry of a caller's effective set.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveEntry {
    pub key: String,
    pub value: FlagValue,
}

/// A caller's whole effective set plus a stable version/ETag.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveSet {
    pub entries: Vec<EffectiveEntry>,
    pub version: String,
}

/// The acting admin for a write (identity already authenticated by the gRPC
/// interceptor; `is_admin` already asserted by the handler guard).
#[derive(Debug, Clone)]
pub struct Actor {
    pub user_id: String,
    /// The caller's token audience (app).
    pub app: String,
    pub is_admin: bool,
}

/// Default L1 TTL backstop (design open-question default).
pub const DEFAULT_TTL: Duration = Duration::from_secs(15);

/// The flags/config service.
pub struct FlagService {
    registry: Registry,
    /// `None` = defaults-only mode (no DB): everything resolves to code defaults,
    /// and admin writes are refused (`FailedPrecondition`).
    store: Option<Arc<dyn FlagStore>>,
    bus: Arc<dyn InvalidationBus>,
    resolver: Arc<dyn AdminScopeResolver>,
    snapshot: RwLock<Arc<Snapshot>>,
    ttl: Duration,
}

impl FlagService {
    pub fn new(
        registry: Registry,
        store: Option<Arc<dyn FlagStore>>,
        bus: Arc<dyn InvalidationBus>,
        resolver: Arc<dyn AdminScopeResolver>,
    ) -> Self {
        Self {
            registry,
            store,
            bus,
            resolver,
            snapshot: RwLock::new(Arc::new(Snapshot::default())),
            ttl: DEFAULT_TTL,
        }
    }

    /// Override the L1 TTL backstop (mainly for tests / tuning).
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    fn snapshot(&self) -> Arc<Snapshot> {
        self.snapshot.read().unwrap().clone()
    }

    /// Reload the L1 snapshot from the store, atomically swapping it in. On error
    /// (Postgres unreachable) the previous snapshot is KEPT — never cleared — so a
    /// transient outage never drops to an empty set mid-flight; a never-loaded
    /// snapshot resolves to code defaults. Defaults-only mode is a no-op.
    pub async fn refresh(&self) -> Result<()> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let all = store.load_all().await?;
        let mut overrides = HashMap::with_capacity(all.len());
        for ov in all {
            overrides.insert((ov.app.clone(), ov.key.clone()), ov);
        }
        let snap = Arc::new(Snapshot {
            overrides,
            loaded_at: Some(Instant::now()),
        });
        *self.snapshot.write().unwrap() = snap;
        Ok(())
    }

    /// Refresh only if the snapshot is older than the TTL (the backstop ticker).
    pub async fn refresh_if_stale(&self) -> Result<()> {
        let stale = self
            .snapshot()
            .loaded_at
            .map(|t| t.elapsed() >= self.ttl)
            .unwrap_or(true);
        if stale { self.refresh().await } else { Ok(()) }
    }

    /// The applicable override for a declared key in this context, if one exists,
    /// applies to the caller's app, matches the declared type, and its rollout
    /// reaches the caller. Otherwise `None` (⇒ the code default is used).
    fn applicable_override(&self, def: &KeyDef, ctx: &EvalContext) -> Option<FlagValue> {
        if !ctx.app_matches(def.app) {
            return None;
        }
        let snap = self.snapshot();
        let ov = snap
            .overrides
            .get(&(def.app.to_string(), def.key.to_string()))?;
        if ov.value_type != def.value_type || !ctx.rollout_reaches(ov.rollout) {
            return None;
        }
        Some(ov.value.clone())
    }

    // --- OpenFeature-shaped typed evaluation (design D8) --------------------

    /// Evaluate a boolean flag: the applicable override, else `default`.
    pub fn bool(&self, key: &str, default: bool, ctx: &EvalContext) -> bool {
        match self.registry.get_by_key(key) {
            Some(def) if def.value_type == ValueType::Bool => self
                .applicable_override(def, ctx)
                .and_then(|v| v.as_bool())
                .unwrap_or(default),
            _ => default,
        }
    }

    /// Evaluate an integer config value.
    pub fn int(&self, key: &str, default: i64, ctx: &EvalContext) -> i64 {
        match self.registry.get_by_key(key) {
            Some(def) if def.value_type == ValueType::Int => self
                .applicable_override(def, ctx)
                .and_then(|v| v.as_i64())
                .unwrap_or(default),
            _ => default,
        }
    }

    /// Evaluate a numeric config value.
    pub fn number(&self, key: &str, default: f64, ctx: &EvalContext) -> f64 {
        match self.registry.get_by_key(key) {
            Some(def) if def.value_type == ValueType::Number => self
                .applicable_override(def, ctx)
                .and_then(|v| v.as_f64())
                .unwrap_or(default),
            _ => default,
        }
    }

    /// Evaluate a string config value.
    pub fn string(&self, key: &str, default: &str, ctx: &EvalContext) -> String {
        match self.registry.get_by_key(key) {
            Some(def) if def.value_type == ValueType::String => self
                .applicable_override(def, ctx)
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| default.to_string()),
            _ => default.to_string(),
        }
    }

    /// Evaluate a JSON config value.
    pub fn json(
        &self,
        key: &str,
        default: serde_json::Value,
        ctx: &EvalContext,
    ) -> serde_json::Value {
        match self.registry.get_by_key(key) {
            Some(def) if def.value_type == ValueType::Json => self
                .applicable_override(def, ctx)
                .and_then(|v| v.as_json().cloned())
                .unwrap_or(default),
            _ => default,
        }
    }

    // --- effective set for the client read ---------------------------------

    /// The caller's whole effective set (every declared key applicable to the
    /// caller's app, resolved for their rollout scope) plus a stable version.
    pub fn effective(&self, ctx: &EvalContext) -> EffectiveSet {
        let mut entries: Vec<EffectiveEntry> = self
            .registry
            .all()
            .iter()
            .filter(|def| ctx.app_matches(def.app))
            .map(|def| EffectiveEntry {
                key: def.key.to_string(),
                value: self
                    .applicable_override(def, ctx)
                    .unwrap_or_else(|| def.default.clone()),
            })
            .collect();
        entries.sort_by(|a, b| a.key.cmp(&b.key));
        let version = version_of(&entries);
        EffectiveSet { entries, version }
    }

    // --- admin surface ------------------------------------------------------

    /// Resolved state of every declared key visible to `actor_app` (its own +
    /// shared `all` keys), optionally narrowed to one app.
    pub fn definitions(&self, actor_app: &str, app_filter: Option<&str>) -> Vec<KeyState> {
        self.registry
            .all()
            .iter()
            .filter(|def| def.app == crate::context::APP_ALL || def.app == actor_app)
            .filter(|def| app_filter.is_none_or(|a| a.is_empty() || def.app == a))
            .map(|def| self.key_state(def))
            .collect()
    }

    /// Like [`definitions`], but also resolves, per key, whether `actor` may edit
    /// it — a per-app admin can change only its own app's keys; `all`/other-app
    /// keys need a platform admin. Costs at most one resolver call. The UI uses
    /// this to hide controls; the backend still enforces on write.
    pub async fn definitions_for(
        &self,
        actor: &Actor,
        app_filter: Option<&str>,
    ) -> Result<Vec<(KeyState, bool)>> {
        let platform = if actor.is_admin {
            self.resolver
                .is_platform_admin(&actor.user_id)
                .await
                .unwrap_or(false)
        } else {
            false
        };
        Ok(self
            .definitions(&actor.app, app_filter)
            .into_iter()
            .map(|ks| {
                let editable = actor.is_admin && (actor.app == ks.def.app || platform);
                (ks, editable)
            })
            .collect())
    }

    fn key_state(&self, def: &KeyDef) -> KeyState {
        let snap = self.snapshot();
        match snap
            .overrides
            .get(&(def.app.to_string(), def.key.to_string()))
        {
            Some(o) => KeyState {
                def: def.clone(),
                effective: o.value.clone(),
                has_override: true,
                rollout: o.rollout,
                updated_by: Some(o.updated_by.clone()),
                updated_at: Some(o.updated_at.to_rfc3339()),
            },
            None => KeyState {
                def: def.clone(),
                effective: def.default.clone(),
                has_override: false,
                rollout: def.rollout,
                updated_by: None,
                updated_at: None,
            },
        }
    }

    /// Set (or replace) an override for a declared key. Enforces: declared-only,
    /// type match, admin + app scoping (per-app admin only its own app; `all` or
    /// another app needs the platform admin), sensitive-key confirmation. Writes
    /// the override + audit, refreshes L1, and publishes an invalidation.
    pub async fn set_value(
        &self,
        actor: &Actor,
        app: &str,
        key: &str,
        value: FlagValue,
        rollout: Option<RolloutScope>,
        confirm: bool,
    ) -> Result<KeyState> {
        let store = self
            .store
            .as_ref()
            .ok_or_else(|| AppError::FailedPrecondition("flag store not configured".into()))?;

        let def = self
            .registry
            .get(app, key)
            .ok_or_else(|| AppError::NotFound(format!("no declared key `{key}` for app `{app}`")))?
            .clone();

        if value.value_type() != def.value_type {
            return Err(AppError::InvalidArgument(format!(
                "value is not a {}",
                def.value_type.as_str()
            )));
        }

        self.authorize_write(actor, &def).await?;

        if def.sensitive && !confirm {
            return Err(AppError::FailedPrecondition(format!(
                "key `{key}` is sensitive and requires explicit confirmation"
            )));
        }

        // Preserve an existing override's rollout unless the caller sets one.
        let prev = self
            .snapshot()
            .overrides
            .get(&(def.app.to_string(), def.key.to_string()))
            .cloned();
        let rollout = rollout
            .or_else(|| prev.as_ref().map(|p| p.rollout))
            .unwrap_or(def.rollout);
        let old_display = prev.as_ref().map(|p| p.value.display());

        store
            .upsert(&OverrideWrite {
                app: def.app.to_string(),
                key: def.key.to_string(),
                value_type: def.value_type,
                value: value.clone(),
                rollout,
                sensitive: def.sensitive,
                actor: actor.user_id.clone(),
                old_display,
            })
            .await?;

        self.refresh().await?;
        self.publish_invalidation().await;

        Ok(KeyState {
            def,
            effective: value,
            has_override: true,
            rollout,
            updated_by: Some(actor.user_id.clone()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        })
    }

    /// Clear a key's override, reverting it to the code default (audited).
    pub async fn clear_value(
        &self,
        actor: &Actor,
        app: &str,
        key: &str,
        confirm: bool,
    ) -> Result<KeyState> {
        let store = self
            .store
            .as_ref()
            .ok_or_else(|| AppError::FailedPrecondition("flag store not configured".into()))?;
        let def = self
            .registry
            .get(app, key)
            .ok_or_else(|| AppError::NotFound(format!("no declared key `{key}` for app `{app}`")))?
            .clone();
        self.authorize_write(actor, &def).await?;
        if def.sensitive && !confirm {
            return Err(AppError::FailedPrecondition(format!(
                "key `{key}` is sensitive and requires explicit confirmation"
            )));
        }
        let prev = self
            .snapshot()
            .overrides
            .get(&(def.app.to_string(), def.key.to_string()))
            .cloned();
        if let Some(prev) = prev {
            store
                .clear(def.app, def.key, &actor.user_id, &prev.value.display())
                .await?;
            self.refresh().await?;
            self.publish_invalidation().await;
        }
        Ok(self.key_state(&def))
    }

    /// Recent change audit (newest first), optionally filtered by app and/or a
    /// single key (the per-parameter drawer passes the key).
    pub async fn recent_changes(
        &self,
        app_filter: Option<&str>,
        key_filter: Option<&str>,
        limit: i64,
    ) -> Result<Vec<crate::store::ChangeRecord>> {
        let store = self
            .store
            .as_ref()
            .ok_or_else(|| AppError::FailedPrecondition("flag store not configured".into()))?;
        store
            .recent_changes(
                app_filter.unwrap_or(""),
                key_filter.unwrap_or(""),
                limit.clamp(1, 500),
            )
            .await
    }

    /// Authorize a write: must be admin; changing an `all` key or another app's key
    /// requires the platform (global) admin, resolved from the account's scoped
    /// roles. A per-app admin may change only its own app's keys.
    async fn authorize_write(&self, actor: &Actor, def: &KeyDef) -> Result<()> {
        if !actor.is_admin {
            return Err(AppError::PermissionDenied("requires role `admin`".into()));
        }
        let needs_platform = def.app == crate::context::APP_ALL || def.app != actor.app;
        if needs_platform && !self.resolver.is_platform_admin(&actor.user_id).await? {
            return Err(AppError::PermissionDenied(format!(
                "changing `{}`-scoped keys requires a platform admin",
                def.app
            )));
        }
        Ok(())
    }

    async fn publish_invalidation(&self) {
        if let Err(e) = self.bus.publish().await {
            tracing::warn!("flag invalidation publish failed: {e}");
        }
    }
}

/// A stable hash of an effective set: identical content ⇒ identical version across
/// every instance (SipHash with a fixed seed via `DefaultHasher::new`).
fn version_of(entries: &[EffectiveEntry]) -> String {
    let mut h = DefaultHasher::new();
    for e in entries {
        e.key.hash(&mut h);
        e.value.value_type().as_str().hash(&mut h);
        e.value.display().hash(&mut h);
    }
    format!("{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::APP_ALL;
    use crate::invalidation::{NoopBus, RecordingBus};
    use crate::registry::{self, APP_MUSIC};
    use crate::resolver::MockAdminScopeResolver;
    use crate::store::{MockFlagStore, StoredOverride};
    use chrono::Utc;

    fn ov(app: &str, key: &str, value: FlagValue, rollout: RolloutScope) -> StoredOverride {
        StoredOverride {
            app: app.into(),
            key: key.into(),
            value_type: value.value_type(),
            value,
            rollout,
            sensitive: false,
            updated_by: "00000000-0000-0000-0000-000000000001".into(),
            updated_at: Utc::now(),
        }
    }

    fn resolver(platform: bool) -> Arc<MockAdminScopeResolver> {
        let mut r = MockAdminScopeResolver::new();
        r.expect_is_platform_admin()
            .returning(move |_| Ok(platform));
        Arc::new(r)
    }

    /// Build a service whose store returns exactly `overrides` from `load_all`.
    async fn service_with(overrides: Vec<StoredOverride>) -> FlagService {
        let mut store = MockFlagStore::new();
        store
            .expect_load_all()
            .returning(move || Ok(overrides.clone()));
        let svc = FlagService::new(
            Registry::default(),
            Some(Arc::new(store)),
            Arc::new(NoopBus),
            resolver(false),
        );
        svc.refresh().await.unwrap();
        svc
    }

    fn user_ctx(app: &str) -> EvalContext {
        EvalContext::anonymous(app)
    }
    fn staff_ctx(app: &str) -> EvalContext {
        EvalContext::authenticated(app, &["admin".into()])
    }

    #[tokio::test]
    async fn absent_key_uses_code_default() {
        let svc = service_with(vec![]).await;
        // no override ⇒ registry default (false / 5 / 2.0)
        assert!(!svc.bool(registry::RATING_ENABLED, false, &user_ctx(APP_MUSIC)));
        assert_eq!(
            svc.int(registry::RATING_REVIEW_MIN_VOTES, 5, &user_ctx(APP_MUSIC)),
            5
        );
        assert_eq!(
            svc.number(registry::RATING_REVIEW_THRESHOLD, 2.0, &user_ctx(APP_MUSIC)),
            2.0
        );
    }

    #[tokio::test]
    async fn stored_value_overrides_default() {
        let svc = service_with(vec![
            ov(
                APP_MUSIC,
                registry::RATING_ENABLED,
                FlagValue::Bool(true),
                RolloutScope::Global,
            ),
            ov(
                APP_MUSIC,
                registry::RATING_REVIEW_MIN_VOTES,
                FlagValue::Int(9),
                RolloutScope::Global,
            ),
        ])
        .await;
        assert!(svc.bool(registry::RATING_ENABLED, false, &user_ctx(APP_MUSIC)));
        assert_eq!(
            svc.int(registry::RATING_REVIEW_MIN_VOTES, 5, &user_ctx(APP_MUSIC)),
            9
        );
    }

    #[tokio::test]
    async fn store_outage_falls_back_to_defaults() {
        // load_all errors ⇒ refresh keeps the (empty) snapshot ⇒ code defaults.
        let mut store = MockFlagStore::new();
        store
            .expect_load_all()
            .returning(|| Err(AppError::Internal(anyhow::anyhow!("db down"))));
        let svc = FlagService::new(
            Registry::default(),
            Some(Arc::new(store)),
            Arc::new(NoopBus),
            resolver(false),
        );
        assert!(svc.refresh().await.is_err());
        // kill-switch stays in its safe (disabled) state despite the outage.
        assert!(!svc.bool(registry::RATING_ENABLED, false, &user_ctx(APP_MUSIC)));
    }

    #[tokio::test]
    async fn defaults_only_mode_when_no_store() {
        let svc = FlagService::new(
            Registry::default(),
            None,
            Arc::new(NoopBus),
            resolver(false),
        );
        svc.refresh().await.unwrap(); // no-op
        assert_eq!(
            svc.int(
                registry::LEADERBOARD_GLOBAL_BEST_N,
                20,
                &user_ctx(APP_MUSIC)
            ),
            20
        );
    }

    #[tokio::test]
    async fn app_scope_isolates_other_apps() {
        // A music-scoped override never applies to a `live` caller.
        let svc = service_with(vec![ov(
            APP_MUSIC,
            registry::RATING_ENABLED,
            FlagValue::Bool(true),
            RolloutScope::Global,
        )])
        .await;
        assert!(svc.bool(registry::RATING_ENABLED, false, &user_ctx(APP_MUSIC)));
        assert!(!svc.bool(registry::RATING_ENABLED, false, &user_ctx("live")));
    }

    #[tokio::test]
    async fn shared_all_key_applies_everywhere() {
        let svc = service_with(vec![ov(
            APP_ALL,
            registry::PLATFORM_MAINTENANCE,
            FlagValue::Bool(true),
            RolloutScope::Global,
        )])
        .await;
        assert!(svc.bool(registry::PLATFORM_MAINTENANCE, false, &user_ctx(APP_MUSIC)));
        assert!(svc.bool(registry::PLATFORM_MAINTENANCE, false, &user_ctx("live")));
    }

    #[tokio::test]
    async fn staff_only_override_reaches_only_staff() {
        let svc = service_with(vec![ov(
            APP_MUSIC,
            registry::LEADERBOARD_GLOBAL_ENABLED,
            FlagValue::Bool(true),
            RolloutScope::StaffOnly,
        )])
        .await;
        assert!(svc.bool(
            registry::LEADERBOARD_GLOBAL_ENABLED,
            false,
            &staff_ctx(APP_MUSIC)
        ));
        assert!(!svc.bool(
            registry::LEADERBOARD_GLOBAL_ENABLED,
            false,
            &user_ctx(APP_MUSIC)
        ));
    }

    #[tokio::test]
    async fn effective_set_is_app_scoped_and_versioned() {
        let svc = service_with(vec![]).await;
        let music = svc.effective(&user_ctx(APP_MUSIC));
        // includes `all` + music keys, excludes nothing live-only (none declared)
        assert!(
            music
                .entries
                .iter()
                .any(|e| e.key == registry::PLATFORM_MAINTENANCE)
        );
        assert!(
            music
                .entries
                .iter()
                .any(|e| e.key == registry::RATING_ENABLED)
        );
        // a live caller does NOT get the music-only keys
        let live = svc.effective(&user_ctx("live"));
        assert!(
            live.entries
                .iter()
                .any(|e| e.key == registry::PLATFORM_MAINTENANCE)
        );
        assert!(
            !live
                .entries
                .iter()
                .any(|e| e.key == registry::RATING_ENABLED)
        );
        assert!(!music.version.is_empty());
    }

    #[tokio::test]
    async fn version_changes_with_content_and_is_stable() {
        let a = service_with(vec![]).await.effective(&user_ctx(APP_MUSIC));
        let b = service_with(vec![]).await.effective(&user_ctx(APP_MUSIC));
        assert_eq!(a.version, b.version, "same content ⇒ same version");
        let c = service_with(vec![ov(
            APP_MUSIC,
            registry::RATING_ENABLED,
            FlagValue::Bool(true),
            RolloutScope::Global,
        )])
        .await
        .effective(&user_ctx(APP_MUSIC));
        assert_ne!(a.version, c.version, "changed content ⇒ changed version");
    }

    fn actor(app: &str, is_admin: bool) -> Actor {
        Actor {
            user_id: "00000000-0000-0000-0000-0000000000aa".into(),
            app: app.into(),
            is_admin,
        }
    }

    /// A service whose store records the upsert it receives.
    fn writable_service(
        bus: Arc<RecordingBus>,
        resolver: Arc<MockAdminScopeResolver>,
        expect: impl Fn(&OverrideWrite) -> bool + Send + 'static,
    ) -> FlagService {
        let mut store = MockFlagStore::new();
        store.expect_load_all().returning(|| Ok(vec![]));
        store
            .expect_upsert()
            .withf(move |w| expect(w))
            .returning(|_| Ok(()));
        FlagService::new(Registry::default(), Some(Arc::new(store)), bus, resolver)
    }

    #[tokio::test]
    async fn set_flag_writes_audit_and_invalidates() {
        let bus = Arc::new(RecordingBus::default());
        let svc = writable_service(bus.clone(), resolver(false), |w| {
            w.key == registry::RATING_ENABLED
                && w.value == FlagValue::Bool(true)
                && w.old_display.is_none() // first set ⇒ no prior value
        });
        let out = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_MUSIC,
                registry::RATING_ENABLED,
                FlagValue::Bool(true),
                None,
                false,
            )
            .await
            .unwrap();
        assert!(out.has_override);
        assert_eq!(out.effective, FlagValue::Bool(true));
        assert_eq!(bus.count(), 1, "edit publishes an invalidation");
    }

    #[tokio::test]
    async fn per_app_admin_cannot_change_shared_or_other_app() {
        // resolver says NOT a platform admin.
        let svc = FlagService::new(
            Registry::default(),
            Some(Arc::new({
                let mut s = MockFlagStore::new();
                s.expect_load_all().returning(|| Ok(vec![]));
                s
            })),
            Arc::new(NoopBus),
            resolver(false),
        );
        // an `all`-scoped key
        let err = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_ALL,
                registry::PLATFORM_MAINTENANCE,
                FlagValue::Bool(true),
                None,
                false,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::PermissionDenied(_)));
    }

    #[tokio::test]
    async fn platform_admin_can_change_shared_key() {
        let svc = writable_service(Arc::new(RecordingBus::default()), resolver(true), |w| {
            w.app == APP_ALL && w.key == registry::PLATFORM_MAINTENANCE
        });
        let out = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_ALL,
                registry::PLATFORM_MAINTENANCE,
                FlagValue::Bool(true),
                Some(RolloutScope::Global),
                false,
            )
            .await
            .unwrap();
        assert!(out.has_override);
    }

    #[tokio::test]
    async fn per_app_admin_can_change_own_app_key() {
        let svc = writable_service(
            Arc::new(RecordingBus::default()),
            resolver(false), // not platform — but own-app edit doesn't need it
            |w| w.app == APP_MUSIC && w.key == registry::REWARDS_ENABLED,
        );
        let out = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_MUSIC,
                registry::REWARDS_ENABLED,
                FlagValue::Bool(true),
                None,
                false,
            )
            .await
            .unwrap();
        assert!(out.has_override);
    }

    #[tokio::test]
    async fn non_admin_is_rejected() {
        let svc = service_with(vec![]).await;
        let err = svc
            .set_value(
                &actor(APP_MUSIC, false),
                APP_MUSIC,
                registry::REWARDS_ENABLED,
                FlagValue::Bool(true),
                None,
                false,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::PermissionDenied(_)));
    }

    #[tokio::test]
    async fn undeclared_key_is_not_editable() {
        let svc = service_with(vec![]).await;
        let err = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_MUSIC,
                "made.up.key",
                FlagValue::Bool(true),
                None,
                false,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn wrong_type_is_rejected() {
        let svc = service_with(vec![]).await;
        let err = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_MUSIC,
                registry::RATING_REVIEW_MIN_VOTES, // int
                FlagValue::Bool(true),
                None,
                false,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn sensitive_key_requires_confirmation() {
        // platform admin (all-scoped sensitive key), but confirm=false ⇒ refused.
        let svc = FlagService::new(
            Registry::default(),
            Some(Arc::new({
                let mut s = MockFlagStore::new();
                s.expect_load_all().returning(|| Ok(vec![]));
                s
            })),
            Arc::new(NoopBus),
            resolver(true),
        );
        let err = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_ALL,
                registry::ACCOUNT_MIN_PUBLIC_SHARING_AGE,
                FlagValue::Int(18),
                None,
                false, // no confirmation
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));
    }

    #[tokio::test]
    async fn sensitive_key_with_confirmation_succeeds() {
        let svc = writable_service(Arc::new(RecordingBus::default()), resolver(true), |w| {
            w.key == registry::ACCOUNT_MIN_PUBLIC_SHARING_AGE && w.value == FlagValue::Int(18)
        });
        let out = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_ALL,
                registry::ACCOUNT_MIN_PUBLIC_SHARING_AGE,
                FlagValue::Int(18),
                None,
                true,
            )
            .await
            .unwrap();
        assert!(out.has_override);
    }

    #[tokio::test]
    async fn set_refused_without_store() {
        let svc = FlagService::new(Registry::default(), None, Arc::new(NoopBus), resolver(true));
        let err = svc
            .set_value(
                &actor(APP_MUSIC, true),
                APP_MUSIC,
                registry::REWARDS_ENABLED,
                FlagValue::Bool(true),
                None,
                false,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)));
    }

    #[tokio::test]
    async fn definitions_for_marks_editability_by_scope() {
        // per-app admin (resolver=false): own-app keys editable, `all` keys not.
        let svc = service_with(vec![]).await; // resolver(false)
        let defs = svc
            .definitions_for(&actor(APP_MUSIC, true), None)
            .await
            .unwrap();
        let own = defs
            .iter()
            .find(|(k, _)| k.def.key == registry::REWARDS_ENABLED)
            .unwrap();
        assert!(own.1, "own-app key editable by a per-app admin");
        let shared = defs
            .iter()
            .find(|(k, _)| k.def.key == registry::PLATFORM_MAINTENANCE)
            .unwrap();
        assert!(!shared.1, "`all` key not editable by a per-app admin");
        // non-admin: nothing editable
        let none = svc
            .definitions_for(&actor(APP_MUSIC, false), None)
            .await
            .unwrap();
        assert!(none.iter().all(|(_, e)| !e));

        // platform admin: `all` keys become editable
        let plat = FlagService::new(
            Registry::default(),
            Some(Arc::new({
                let mut s = MockFlagStore::new();
                s.expect_load_all().returning(|| Ok(vec![]));
                s
            })),
            Arc::new(NoopBus),
            resolver(true),
        );
        let pdefs = plat
            .definitions_for(&actor(APP_MUSIC, true), None)
            .await
            .unwrap();
        let pshared = pdefs
            .iter()
            .find(|(k, _)| k.def.key == registry::PLATFORM_MAINTENANCE)
            .unwrap();
        assert!(pshared.1, "`all` key editable by a platform admin");
    }

    #[tokio::test]
    async fn definitions_are_app_scoped_and_reflect_overrides() {
        let svc = service_with(vec![ov(
            APP_MUSIC,
            registry::REWARDS_ENABLED,
            FlagValue::Bool(true),
            RolloutScope::StaffOnly,
        )])
        .await;
        let defs = svc.definitions(APP_MUSIC, None);
        let rewards = defs
            .iter()
            .find(|k| k.def.key == registry::REWARDS_ENABLED)
            .unwrap();
        assert!(rewards.has_override);
        assert_eq!(rewards.effective, FlagValue::Bool(true));
        assert_eq!(rewards.rollout, RolloutScope::StaffOnly);
        // a not-overridden key shows its default
        let onboarding = defs
            .iter()
            .find(|k| k.def.key == registry::ONBOARDING_ENABLED)
            .unwrap();
        assert!(!onboarding.has_override);
        assert_eq!(onboarding.effective, FlagValue::Bool(false));
        // app filter to `all` yields only shared keys
        let shared = svc.definitions(APP_MUSIC, Some(APP_ALL));
        assert!(shared.iter().all(|k| k.def.app == APP_ALL));
    }
}
