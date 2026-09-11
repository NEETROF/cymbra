//! `cymbra-platform` — cross-cutting primitives shared by every Cymbra ID module.
//!
//! Per design D3/D9 this crate owns: typed config, structured logging/telemetry,
//! the internal-token JWT codec + interceptor, JWKS publishing, the OIDC/JWKS
//! verification helper, argon2id hashing + password policy, the email-sender port,
//! the Redis client/port + rate-limiter, the gRPC error mapping, and the
//! [`AuthIdentity`] request context. It MUST NOT depend on any module crate.

pub mod cache;
pub mod config;
pub mod db;
pub mod email;
pub mod email_template;
pub mod error;
pub mod guard;
pub mod identity;
pub mod interceptor;
pub mod jwks;
pub mod logging;
pub mod metrics;
pub mod oidc;
pub mod password;
pub mod ratelimit;
pub mod telemetry;
pub mod token;

pub use error::{AppError, Result};
pub use identity::AuthIdentity;

/// The authorization scopes a role can be held in: the `global` break-glass plus
/// each app scope. `global` unions into every scope; the app scopes are isolated
/// from one another (change: scope-aware-role-admin, mirrors the user module's
/// `SCOPES`).
pub const SCOPES: [&str; 4] = [GLOBAL_SCOPE, MUSIC_SCOPE, LIVE_SCOPE, LINGUA_SCOPE];

/// The break-glass scope: a role held here applies in every scope.
pub const GLOBAL_SCOPE: &str = "global";
/// The Cymbra Music product scope. Spelled once so "where is music authority
/// checked?" is a search for this name rather than for a string literal, and so a
/// second product cannot inherit a music gate by accident.
pub const MUSIC_SCOPE: &str = "music";
/// The Cymbra Live product scope.
pub const LIVE_SCOPE: &str = "live";
/// The Cymbra Lingua product scope (change: add-lingua-back-office). Its admin gates
/// the Lingua ops console; a `music`/`live` admin does not inherit it.
pub const LINGUA_SCOPE: &str = "lingua";

/// The app scopes an administrator can be scoped to (everything in [`SCOPES`]
/// except the `global` break-glass) — the audiences the back-office session
/// aggregates.
pub const APP_SCOPES: [&str; 3] = [MUSIC_SCOPE, LIVE_SCOPE, LINGUA_SCOPE];

/// The dedicated audience the back office authenticates against: its access token
/// carries the administrator's real roles across `global ∪ music ∪ live`, so one
/// session can administer every scope they are entitled to (change:
/// scope-aware-role-admin). App clients keep using their single app audience.
pub const BACKOFFICE_AUDIENCE: &str = "back-office";
