//! `cymbra-feature-flags` — Cymbra's shared, app-agnostic runtime feature-flag &
//! config store (change: add-runtime-feature-flags).
//!
//! A dedicated platform crate on the model of `cymbra-jobs`, reused by the server,
//! the worker, and every app (music, live, future) — NOT part of `cymbra-music`.
//! It owns its `feature_flags` schema/migrations and exposes:
//!
//! - a **code [`registry`]** of declared keys (typed defaults, app scope, rollout
//!   intent, sensitivity) — the DB only overrides declared keys;
//! - an in-house, **OpenFeature-shaped** typed evaluation API on [`FlagService`]
//!   (`bool`/`int`/`number`/`string`/`json`) with an **L1 in-process snapshot**
//!   (hot-path reads) refreshed on a TTL backstop or an [`invalidation`] ping;
//! - per-app + rollout **scope resolution**, fail-safe fallback to code defaults;
//! - the admin surface (set/clear/list + audit) with platform/per-app scoping; and
//! - the gRPC [`grpc::FlagGrpc`] adapter for the client read + admin edits.
//!
//! The pure modules (`value`, `context`, `registry`, `service`) are host-tested;
//! the I/O glue (`store::PgFlagStore`, the Redis parts of `invalidation`) sits
//! behind seams and is coverage-excluded like the other adapters.

pub mod context;
pub mod grpc;
pub mod invalidation;
pub mod registry;
pub mod resolver;
pub mod service;
pub mod store;
pub mod value;

pub use context::{APP_ALL, EvalContext, NoPlanContext, PlanContextSource, RolloutScope};
pub use invalidation::{
    DEFAULT_CHANNEL, InvalidationBus, NoopBus, RedisInvalidationBus, run_invalidation_listener,
};
pub use registry::{KeyDef, Registry, builtin};
pub use resolver::AdminScopeResolver;
pub use service::{Actor, EffectiveEntry, EffectiveSet, FlagService, KeyState};
pub use store::{ChangeRecord, FlagStore, OverrideWrite, PgFlagStore, StoredOverride};
pub use value::{FlagValue, ValueType};

/// Generated `cymbra.flags.v1` protobuf types + tonic client/server stubs.
// `tonic::Status` is large by design; newer clippy flags every generated
// client/server signature for it.
#[allow(clippy::result_large_err)]
pub mod proto {
    tonic::include_proto!("cymbra.flags.v1");
}

/// Embedded migrations for the `feature_flags` schema, run by whichever binary
/// owns the flags pool (the server) at startup via `MIGRATOR.run(&pool)`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
