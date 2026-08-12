//! `cymbra-notifications` — the server-driven push platform (change:
//! add-push-notifications).
//!
//! Reusable infrastructure, **not** a notification type: features declare their
//! own types on top of it (design D6). It provides
//!
//! - the [`grpc::NotificationGrpc`] `NotificationService` adapter — device-token
//!   register/unregister, per-category consent, timezone;
//! - the [`repo::PushRegistry`] port (mockall-doubled) over the push tables in the
//!   `user_account` schema, with the Postgres implementation in [`pg`];
//! - the pure, host-testable [`select_core`] — every "who receives this" gate
//!   (kill-switch, category enable, opt-out, local hour, token presence);
//! - the [`sender::PushSender`] port (mockall-doubled) and its single FCM HTTP v1
//!   implementation in [`fcm`], which reaches iOS + Android + macOS; and
//! - [`dispatch::Dispatcher`], the selection → send → prune loop the worker's
//!   `push_dispatch` job runs.
//!
//! It owns **no** schema: device tokens, category preferences and the timezone are
//! account data living in `user_account` (see the user module's
//! `0008_push_notifications.sql`), so account erasure cascades them away.
//!
//! # Adding a notification type
//!
//! See `README.md` — a feature supplies a category id, two feature-flag keys, a
//! candidate query and a message; the platform core is untouched.

pub mod dispatch;
pub mod fcm;
pub mod grpc;
pub mod pg;
pub mod repo;
pub mod select_core;
pub mod sender;

pub use dispatch::{DispatchReport, Dispatcher, resolve_flags};
pub use fcm::{FcmSender, ServiceAccount};
pub use grpc::NotificationGrpc;
pub use pg::PgPushRegistry;
pub use repo::{Audience, Candidate, CategoryPref, Platform, PushRegistry};
pub use select_core::{DEFAULT_TIMEZONE, SelectionFlags, select_recipients, validate_timezone};
pub use sender::{PushMessage, PushSender, SendOutcome};

/// Generated `cymbra.notifications.v1` protobuf messages + tonic client/server
/// stubs (the NotificationService).
pub mod proto {
    tonic::include_proto!("cymbra.notifications.v1");
}
