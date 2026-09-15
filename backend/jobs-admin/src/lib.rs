//! `cymbra-jobs-admin` — the back-office Jobs console over the job queue (change:
//! add-admin-jobs-console).
//!
//! A separate crate from `cymbra-jobs` on purpose. The substrate stays free of
//! `cymbra-platform` so the engine remains extractable (see `cymbra_jobs::error`);
//! this console is transport — gRPC, the admin session, `AppError` — and consumes the
//! substrate like any other product would. It exposes:
//!
//! - [`admin_grpc::JobsAdminGrpc`], the `JobsAdminService` adapter, gated on
//!   `global/admin`;
//! - [`admin::JobsAdminModule`] over the consumer-declared [`admin::JobsAdminRepo`]
//!   port (mockall-doubled), with its pure rules in [`admin_core`];
//! - [`pg_admin::PgJobsAdminRepo`], which only calls the `jobs.admin_*` SECURITY
//!   DEFINER functions as `jobs_admin_svc` — a role that cannot read a job payload.

pub mod admin;
pub mod admin_core;
pub mod admin_grpc;
pub mod pg_admin;

pub use admin::{JobsAdminModule, JobsAdminRepo, JobsPage, PeriodRow};
pub use admin_grpc::JobsAdminGrpc;
pub use pg_admin::PgJobsAdminRepo;

/// Generated `cymbra.jobs.v1` protobuf messages + the tonic server stub.
// `tonic::Status` is large by design; newer clippy flags every generated
// server signature for it.
#[allow(clippy::result_large_err)]
pub mod proto {
    tonic::include_proto!("cymbra.jobs.v1");
}
