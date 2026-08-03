// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The `UsageService` gRPC **server** adapter (change: add-feature-usage-analytics,
//! task 4.4). Authenticates the caller (identity from the internal-token
//! interceptor, never the body), maps them to a period-salted pseudonymous
//! `user_bucket`, validates each event of the batch — skipping malformed ones
//! without failing the batch (design D5/D7) — and persists the valid rows. Returns
//! accepted/skipped counts; ingestion is best-effort, so the client drops its
//! flushed batch on any success.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::proto::{
    ActionCount as ProtoActionCount, DeviceClassUsers as ProtoDeviceClassUsers,
    GetActionBreakdownRequest, GetActionBreakdownResponse, GetUsersSummaryRequest,
    GetUsersSummaryResponse, ListActionsRequest, ListActionsResponse,
    PlatformUsers as ProtoPlatformUsers, ReportEventsRequest, ReportEventsResponse,
    UsageQuery as ProtoUsageQuery,
    usage_service_server::{UsageService, UsageServiceServer},
};
use crate::read::{UsageQuery, UsageReadRepo};
use crate::repo::{UsageEventRepo, UsageRow};
use crate::usage_core::{RawEvent, user_bucket, validate};

/// Wraps the raw-event repo, the reporting reads, and the bucketing master secret
/// as a tonic `UsageService`.
pub struct UsageGrpc {
    repo: Arc<dyn UsageEventRepo>,
    read: Arc<dyn UsageReadRepo>,
    /// Master secret for the period-salted `user_bucket` (design D2, Option A).
    master_secret: Arc<[u8]>,
}

impl UsageGrpc {
    pub fn new(
        repo: Arc<dyn UsageEventRepo>,
        read: Arc<dyn UsageReadRepo>,
        master_secret: impl Into<Arc<[u8]>>,
    ) -> Self {
        Self {
            repo,
            read,
            master_secret: master_secret.into(),
        }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> UsageServiceServer<Self> {
        UsageServiceServer::new(self)
    }
}

fn caller<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .map(|id| id.user_id.clone())
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

/// Gate the reporting reads on a music-scope (or global break-glass) admin — the
/// console is a music-app product surface (change: add-feature-usage-analytics,
/// task 7.3).
fn require_music_admin<T>(req: &Request<T>) -> Result<(), Status> {
    let id = req
        .extensions()
        .get::<AuthIdentity>()
        .ok_or_else(|| Status::unauthenticated("missing identity"))?;
    if id.has_role_in_scope("music", "admin") {
        Ok(())
    } else {
        Err(Status::permission_denied("admin scope required"))
    }
}

/// Non-empty optional string (proto3 sends `Some("")` for an unset text filter).
fn some_nonempty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.is_empty())
}

/// Translate a proto query into the domain query, parsing the inclusive day bounds.
fn to_query(p: Option<ProtoUsageQuery>) -> Result<UsageQuery, Status> {
    let p = p.ok_or_else(|| Status::invalid_argument("missing query"))?;
    let parse_day = |s: &str| {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| Status::invalid_argument(format!("invalid day {s:?} (want yyyy-mm-dd)")))
    };
    let from_day = parse_day(&p.from_day)?;
    let to_day = parse_day(&p.to_day)?;
    if from_day > to_day {
        return Err(Status::invalid_argument("from_day is after to_day"));
    }
    Ok(UsageQuery {
        from_day,
        to_day,
        platform: some_nonempty(p.platform),
        device_class: some_nonempty(p.device_class),
        action: some_nonempty(p.action),
    })
}

fn read_err(e: anyhow::Error) -> Status {
    tracing::warn!(error = %e, "usage read failed");
    Status::internal("failed to read usage metrics")
}

#[tonic::async_trait]
impl UsageService for UsageGrpc {
    async fn report_events(
        &self,
        req: Request<ReportEventsRequest>,
    ) -> Result<Response<ReportEventsResponse>, Status> {
        let owner = caller(&req)?;
        let events = req.into_inner().events;
        // One server clock for the whole batch: clamps client skew + salts buckets.
        let received_at = chrono::Utc::now();

        let mut rows: Vec<UsageRow> = Vec::with_capacity(events.len());
        let mut skipped: u32 = 0;
        for ev in events {
            let raw = RawEvent {
                action: ev.action,
                variant: ev.variant,
                subject_id: ev.subject_id,
                platform: ev.platform,
                device_class: ev.device_class,
                app_version: ev.app_version,
                locale: ev.locale,
                occurred_at_ms: ev.occurred_at_ms,
            };
            match validate(&raw, received_at) {
                Ok(valid) => {
                    // Bucket by the event's (clamped) period, not the batch clock,
                    // so a month-boundary buffered event salts into its own period.
                    let bucket = user_bucket(&self.master_secret, &owner, valid.occurred_at);
                    rows.push(UsageRow {
                        user_bucket: bucket,
                        event: valid,
                    });
                }
                Err(_) => skipped += 1,
            }
        }

        let accepted = self.repo.insert_batch(&rows).await.map_err(|e| {
            // Never surface internals; the client retries silently (best-effort).
            tracing::warn!(error = %e, "usage insert_batch failed");
            Status::internal("failed to record events")
        })? as u32;

        Ok(Response::new(ReportEventsResponse { accepted, skipped }))
    }

    async fn get_users_summary(
        &self,
        req: Request<GetUsersSummaryRequest>,
    ) -> Result<Response<GetUsersSummaryResponse>, Status> {
        require_music_admin(&req)?;
        let q = to_query(req.into_inner().query)?;
        let s = self.read.users_summary(&q).await.map_err(read_err)?;
        Ok(Response::new(GetUsersSummaryResponse {
            total_users: s.total_users,
            by_platform: s
                .by_platform
                .into_iter()
                .map(|p| ProtoPlatformUsers {
                    platform: p.platform,
                    users: p.users,
                })
                .collect(),
            by_device_class: s
                .by_device_class
                .into_iter()
                .map(|d| ProtoDeviceClassUsers {
                    device_class: d.device_class,
                    users: d.users,
                })
                .collect(),
        }))
    }

    async fn get_action_breakdown(
        &self,
        req: Request<GetActionBreakdownRequest>,
    ) -> Result<Response<GetActionBreakdownResponse>, Status> {
        require_music_admin(&req)?;
        let q = to_query(req.into_inner().query)?;
        let rows = self.read.action_breakdown(&q).await.map_err(read_err)?;
        Ok(Response::new(GetActionBreakdownResponse {
            rows: rows
                .into_iter()
                .map(|r| ProtoActionCount {
                    action: r.action,
                    variant: r.variant,
                    events: r.events,
                })
                .collect(),
        }))
    }

    async fn list_actions(
        &self,
        req: Request<ListActionsRequest>,
    ) -> Result<Response<ListActionsResponse>, Status> {
        require_music_admin(&req)?;
        let actions = self.read.list_actions().await.map_err(read_err)?;
        Ok(Response::new(ListActionsResponse { actions }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::UsageEvent;
    use crate::read::{MockUsageReadRepo, UsersSummary};
    use crate::repo::MockUsageEventRepo;
    use std::collections::BTreeMap;

    fn valid_event() -> UsageEvent {
        UsageEvent {
            action: "play_start".into(),
            variant: None,
            subject_id: Some("score-1".into()),
            platform: "ios".into(),
            device_class: "phone".into(),
            app_version: "1.17.0".into(),
            locale: "fr".into(),
            occurred_at_ms: chrono::Utc::now().timestamp_millis(),
        }
    }

    fn grpc_with(repo: MockUsageEventRepo, read: MockUsageReadRepo) -> UsageGrpc {
        UsageGrpc::new(Arc::new(repo), Arc::new(read), b"secret".to_vec())
    }

    fn grpc_accepting_all() -> UsageGrpc {
        let mut repo = MockUsageEventRepo::new();
        repo.expect_insert_batch()
            .returning(|rows| Ok(rows.len() as u64));
        grpc_with(repo, MockUsageReadRepo::new())
    }

    fn authed<T>(msg: T, user_id: &str) -> Request<T> {
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: vec!["user".into()],
            ..Default::default()
        });
        req
    }

    /// A request from a music-scope admin (for the reporting reads).
    fn authed_admin<T>(msg: T) -> Request<T> {
        let mut req = Request::new(msg);
        let mut by_scope = BTreeMap::new();
        by_scope.insert("music".to_string(), vec!["admin".to_string()]);
        req.extensions_mut().insert(AuthIdentity {
            user_id: "admin-1".into(),
            audience: "back-office".into(),
            roles: vec!["admin".into()],
            roles_by_scope: by_scope,
        });
        req
    }

    fn query() -> ProtoUsageQuery {
        ProtoUsageQuery {
            from_day: "2026-06-01".into(),
            to_day: "2026-06-30".into(),
            platform: None,
            device_class: None,
            action: None,
        }
    }

    #[tokio::test]
    async fn rejects_unauthenticated() {
        let g = grpc_accepting_all();
        let err = g
            .report_events(Request::new(ReportEventsRequest {
                events: vec![valid_event()],
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn persists_valid_and_reports_counts() {
        let g = grpc_accepting_all();
        let resp = g
            .report_events(authed(
                ReportEventsRequest {
                    events: vec![valid_event(), valid_event()],
                },
                "u1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.accepted, 2);
        assert_eq!(resp.skipped, 0);
    }

    #[tokio::test]
    async fn malformed_event_is_skipped_not_fatal() {
        // The mock asserts exactly one valid row reaches the repo.
        let mut repo = MockUsageEventRepo::new();
        repo.expect_insert_batch()
            .withf(|rows| rows.len() == 1 && rows[0].event.action == "play_start")
            .returning(|rows| Ok(rows.len() as u64));
        let g = grpc_with(repo, MockUsageReadRepo::new());

        let mut bad = valid_event();
        bad.action = "NOT VALID".into();
        let resp = g
            .report_events(authed(
                ReportEventsRequest {
                    events: vec![valid_event(), bad],
                },
                "u1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.accepted, 1);
        assert_eq!(resp.skipped, 1);
    }

    #[tokio::test]
    async fn empty_batch_is_a_no_op() {
        let mut repo = MockUsageEventRepo::new();
        repo.expect_insert_batch()
            .returning(|rows| Ok(rows.len() as u64));
        let g = grpc_with(repo, MockUsageReadRepo::new());
        let resp = g
            .report_events(authed(ReportEventsRequest { events: vec![] }, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.accepted, 0);
        assert_eq!(resp.skipped, 0);
    }

    // --- reporting reads (admin-gated) ----------------------------------------

    #[tokio::test]
    async fn users_summary_requires_admin() {
        let g = grpc_accepting_all();
        // A plain authenticated user is refused.
        let err = g
            .get_users_summary(authed(
                GetUsersSummaryRequest {
                    query: Some(query()),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn reads_reject_unauthenticated() {
        let g = grpc_accepting_all();
        let err = g
            .list_actions(Request::new(ListActionsRequest {}))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn admin_gets_users_summary() {
        let mut read = MockUsageReadRepo::new();
        read.expect_users_summary().returning(|_| {
            Ok(UsersSummary {
                total_users: 7,
                by_platform: vec![],
                by_device_class: vec![],
            })
        });
        let g = grpc_with(MockUsageEventRepo::new(), read);
        let resp = g
            .get_users_summary(authed_admin(GetUsersSummaryRequest {
                query: Some(query()),
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.total_users, 7);
    }

    #[tokio::test]
    async fn admin_lists_actions_data_driven() {
        let mut read = MockUsageReadRepo::new();
        read.expect_list_actions()
            .returning(|| Ok(vec!["auth_sign_in".into(), "play_start".into()]));
        let g = grpc_with(MockUsageEventRepo::new(), read);
        let resp = g
            .list_actions(authed_admin(ListActionsRequest {}))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.actions, vec!["auth_sign_in", "play_start"]);
    }

    #[tokio::test]
    async fn bad_day_is_rejected() {
        let mut read = MockUsageReadRepo::new();
        read.expect_action_breakdown().returning(|_| Ok(vec![]));
        let g = grpc_with(MockUsageEventRepo::new(), read);
        let err = g
            .get_action_breakdown(authed_admin(GetActionBreakdownRequest {
                query: Some(ProtoUsageQuery {
                    from_day: "nope".into(),
                    to_day: "2026-06-30".into(),
                    platform: None,
                    device_class: None,
                    action: None,
                }),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }
}
