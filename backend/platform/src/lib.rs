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
pub mod error;
pub mod guard;
pub mod identity;
pub mod interceptor;
pub mod jwks;
pub mod logging;
pub mod oidc;
pub mod password;
pub mod ratelimit;
pub mod token;

pub use error::{AppError, Result};
pub use identity::AuthIdentity;
