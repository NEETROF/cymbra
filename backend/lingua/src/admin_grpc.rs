// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Tonic adapter for `LinguaAdminService` — thin transport (coverage-excluded); the
//! aggregate shaping is host-tested in `admin_core`/`admin`. Every RPC gates on
//! `admin` in the `lingua` scope (`require_admin_in_scope`, `global/admin` break-glass
//! included), so a console token holding `admin` in another product's scope is refused.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::{LINGUA_SCOPE, guard::require_admin_in_scope};
use tonic::{Request, Response, Status};

use crate::admin::LinguaAdminModule;
use crate::admin_core::SeriesMetric;
use crate::admin_core::iso_day;
use crate::grpc_util::identity;
use crate::proto::lingua_admin_service_server::LinguaAdminService;
use crate::proto::{
    AdminGetLeaveReasonsRequest, AdminGetLeaveReasonsResponse, AdminGetLinguaUsageRequest,
    AdminGetLinguaUsageResponse, AdminGetLinguaUsageSeriesRequest,
    AdminGetLinguaUsageSeriesResponse, AdminGetOnlineTranslationUsageRequest,
    AdminGetOnlineTranslationUsageResponse, AdminGetUsageFunnelRequest,
    AdminGetUsageFunnelResponse, LeaveReasonTotal, LinguaLanguageUsage, LinguaSeriesPoint,
    OnlineTranslationUsageRow, UsageFunnelWeek,
};

pub struct LinguaAdminGrpc {
    module: Arc<LinguaAdminModule>,
}

impl LinguaAdminGrpc {
    pub fn new(module: Arc<LinguaAdminModule>) -> Self {
        Self { module }
    }
}

/// Map the proto metric enum (an i32 on the wire) to the domain metric; an unknown
/// value falls back to words-learned rather than erroring.
fn metric_from_proto(v: i32) -> SeriesMetric {
    match v {
        1 => SeriesMetric::Reviews,
        2 => SeriesMetric::Exposures,
        3 => SeriesMetric::UnknownSeen,
        _ => SeriesMetric::WordsLearned,
    }
}

/// Pull the (from_day, to_day) out of a required window, erroring if it is absent.
fn window(w: Option<crate::proto::LinguaWindow>) -> Result<(String, String), Status> {
    let w = w.ok_or_else(|| Status::invalid_argument("window is required"))?;
    Ok((w.from_day, w.to_day))
}

#[tonic::async_trait]
impl LinguaAdminService for LinguaAdminGrpc {
    async fn admin_get_lingua_usage(
        &self,
        req: Request<AdminGetLinguaUsageRequest>,
    ) -> Result<Response<AdminGetLinguaUsageResponse>, Status> {
        require_admin_in_scope(&identity(&req)?, LINGUA_SCOPE)?;
        let (from, to) = window(req.into_inner().window)?;
        let u = self.module.get_usage(&from, &to).await?;
        Ok(Response::new(AdminGetLinguaUsageResponse {
            active_accounts: u.active_accounts,
            words_learned: u.words_learned,
            reviews: u.reviews,
            words_read: u.words_read,
            new_words_seen: u.new_words_seen,
            by_language: u
                .by_language
                .into_iter()
                .map(|l| LinguaLanguageUsage {
                    language: l.language,
                    active_accounts: l.active_accounts,
                    words_learned: l.words_learned,
                    reviews: l.reviews,
                    words_read: l.words_read,
                    new_words_seen: l.new_words_seen,
                })
                .collect(),
        }))
    }

    async fn admin_get_lingua_usage_series(
        &self,
        req: Request<AdminGetLinguaUsageSeriesRequest>,
    ) -> Result<Response<AdminGetLinguaUsageSeriesResponse>, Status> {
        require_admin_in_scope(&identity(&req)?, LINGUA_SCOPE)?;
        let r = req.into_inner();
        let (from, to) = window(r.window)?;
        let language = if r.language.is_empty() {
            None
        } else {
            Some(r.language.as_str())
        };
        let points = self
            .module
            .get_series(&from, &to, metric_from_proto(r.metric), language)
            .await?;
        Ok(Response::new(AdminGetLinguaUsageSeriesResponse {
            points: points
                .into_iter()
                .map(|p| LinguaSeriesPoint {
                    day: p.day,
                    value: p.value,
                })
                .collect(),
        }))
    }

    async fn admin_get_online_translation_usage(
        &self,
        req: Request<AdminGetOnlineTranslationUsageRequest>,
    ) -> Result<Response<AdminGetOnlineTranslationUsageResponse>, Status> {
        require_admin_in_scope(&identity(&req)?, LINGUA_SCOPE)?;
        let (from, to) = window(req.into_inner().window)?;
        let (rows, totals) = self.module.get_online_translation_usage(&from, &to).await?;
        Ok(Response::new(AdminGetOnlineTranslationUsageResponse {
            rows: rows
                .into_iter()
                .map(|r| OnlineTranslationUsageRow {
                    day: iso_day(r.day),
                    provider: r.provider,
                    pair: r.pair,
                    answered: r.answered,
                    failed: r.failed,
                    limited: r.limited,
                    input_tokens: r.input_tokens,
                    output_tokens: r.output_tokens,
                    cost_micro_usd: r.cost_micro_usd,
                })
                .collect(),
            answered: totals.answered,
            failed: totals.failed,
            limited: totals.limited,
            cost_micro_usd: totals.cost_micro_usd,
        }))
    }

    async fn admin_get_usage_funnel(
        &self,
        req: Request<AdminGetUsageFunnelRequest>,
    ) -> Result<Response<AdminGetUsageFunnelResponse>, Status> {
        require_admin_in_scope(&identity(&req)?, LINGUA_SCOPE)?;
        let r = req.into_inner();
        let (from, to) = window(r.window)?;
        let weeks = self.module.get_usage_funnel(&from, &to, &r.browser).await?;
        Ok(Response::new(AdminGetUsageFunnelResponse {
            weeks: weeks
                .into_iter()
                .map(|w| UsageFunnelWeek {
                    week: iso_day(w.week),
                    installed: w.installed,
                    set_up: w.set_up,
                    first_analysis: w.first_analysis,
                    returned_day_1: w.returned_day_1,
                    returned_days_7_to_13: w.returned_days_7_13,
                    returned_days_30_to_59: w.returned_days_30_59,
                    capped: w.capped,
                })
                .collect(),
        }))
    }

    async fn admin_get_leave_reasons(
        &self,
        req: Request<AdminGetLeaveReasonsRequest>,
    ) -> Result<Response<AdminGetLeaveReasonsResponse>, Status> {
        require_admin_in_scope(&identity(&req)?, LINGUA_SCOPE)?;
        let r = req.into_inner();
        let (from, to) = window(r.window)?;
        let totals = self
            .module
            .get_leave_reasons(&from, &to, &r.browser)
            .await?;
        Ok(Response::new(AdminGetLeaveReasonsResponse {
            reasons: totals
                .reasons
                .into_iter()
                .map(|(reason, count)| LeaveReasonTotal {
                    reason: reason.to_string(),
                    count,
                })
                .collect(),
            capped: totals.capped,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::LinguaAdminRepo;
    use crate::admin_core::Usage;
    use async_trait::async_trait;
    use cymbra_platform::{AuthIdentity, Result};
    use std::collections::BTreeMap;
    use tonic::Code;

    #[derive(Default)]
    struct QuietRepo;

    #[async_trait]
    impl LinguaAdminRepo for QuietRepo {
        async fn usage(&self, _from: i32, _to: i32) -> Result<Usage> {
            Ok(Usage::default())
        }
        async fn series(
            &self,
            _from: i32,
            _to: i32,
            _metric: SeriesMetric,
            _language: Option<&str>,
        ) -> Result<Vec<(i32, i64)>> {
            Ok(vec![])
        }
        async fn online_translation_usage(
            &self,
            _from: i32,
            _to: i32,
        ) -> Result<Vec<crate::admin_core::OnlineUsageRow>> {
            Ok(vec![])
        }
        async fn usage_funnel(
            &self,
            from_week: i32,
            _to_day: i32,
            browser: Option<&str>,
            _ceiling: i64,
        ) -> Result<Vec<crate::admin_core::FunnelRow>> {
            // One Firefox week and one Chromium week, so the filter is visible.
            let rows = [("firefox", from_week, 3), ("chromium", from_week + 7, 5)];
            Ok(rows
                .into_iter()
                .filter(|(b, _, _)| browser.is_none_or(|f| f == *b))
                .map(|(_, week, count)| crate::admin_core::FunnelRow {
                    week,
                    step: "installed".into(),
                    count,
                    capped: false,
                })
                .collect())
        }
        async fn leave_reasons(
            &self,
            _from_week: i32,
            _to_day: i32,
            _browser: Option<&str>,
            _ceiling: i64,
        ) -> Result<Vec<crate::admin_core::ReasonRow>> {
            Ok(vec![crate::admin_core::ReasonRow {
                reason: "unclear".into(),
                count: 2,
                capped: false,
            }])
        }
    }

    fn grpc() -> LinguaAdminGrpc {
        LinguaAdminGrpc::new(Arc::new(LinguaAdminModule::new(Arc::new(QuietRepo))))
    }

    fn with_identity<T>(mut req: Request<T>, scopes: &[(&str, &[&str])]) -> Request<T> {
        let ident = scoped(scopes).extensions().get::<AuthIdentity>().cloned();
        if let Some(i) = ident {
            req.extensions_mut().insert(i);
        }
        req
    }

    fn funnel_request(browser: &str) -> Request<AdminGetUsageFunnelRequest> {
        Request::new(AdminGetUsageFunnelRequest {
            window: Some(crate::proto::LinguaWindow {
                from_day: "2026-10-21".into(),
                to_day: "2026-11-01".into(),
            }),
            browser: browser.into(),
        })
    }

    fn reasons_request() -> Request<AdminGetLeaveReasonsRequest> {
        Request::new(AdminGetLeaveReasonsRequest {
            window: Some(crate::proto::LinguaWindow {
                from_day: "2026-10-21".into(),
                to_day: "2026-11-01".into(),
            }),
            browser: String::new(),
        })
    }

    #[tokio::test]
    async fn the_funnel_answers_weeks_from_the_window_and_the_filter() {
        let admin: &[(&str, &[&str])] = &[("lingua", &["admin"])];
        let all = grpc()
            .admin_get_usage_funnel(with_identity(funnel_request(""), admin))
            .await
            .unwrap()
            .into_inner();
        // The window starts on a Wednesday: its week is the Monday before.
        assert_eq!(
            all.weeks
                .iter()
                .map(|w| w.week.as_str())
                .collect::<Vec<_>>(),
            ["2026-10-19", "2026-10-26"]
        );
        let firefox = grpc()
            .admin_get_usage_funnel(with_identity(funnel_request("firefox"), admin))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(firefox.weeks.len(), 1);
        assert_eq!(firefox.weeks[0].installed, 3);
        let bad = grpc()
            .admin_get_usage_funnel(with_identity(funnel_request("opera"), admin))
            .await
            .unwrap_err();
        assert_eq!(bad.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn the_leave_reasons_list_the_six_answers() {
        let r = grpc()
            .admin_get_leave_reasons(with_identity(reasons_request(), &[("lingua", &["admin"])]))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.reasons.len(), 6);
        assert_eq!(
            (r.reasons[4].reason.as_str(), r.reasons[4].count),
            ("unclear", 2)
        );
    }

    #[tokio::test]
    async fn the_usage_counts_are_refused_to_non_lingua_admins() {
        for scopes in [
            &[("music", &["admin"][..])][..],
            &[("lingua", &["moderator"][..])][..],
            &[][..],
        ] {
            let err = grpc()
                .admin_get_usage_funnel(with_identity(funnel_request(""), scopes))
                .await
                .unwrap_err();
            assert_eq!(err.code(), Code::PermissionDenied);
            let err = grpc()
                .admin_get_leave_reasons(with_identity(reasons_request(), scopes))
                .await
                .unwrap_err();
            assert_eq!(err.code(), Code::PermissionDenied);
        }
        let err = grpc()
            .admin_get_usage_funnel(funnel_request(""))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::Unauthenticated);
    }

    /// A request carrying an identity with the given `scope -> roles`.
    fn scoped(scopes: &[(&str, &[&str])]) -> Request<AdminGetLinguaUsageRequest> {
        let mut by_scope = BTreeMap::new();
        for (scope, roles) in scopes {
            by_scope.insert(
                scope.to_string(),
                roles.iter().map(|r| r.to_string()).collect(),
            );
        }
        let mut req = Request::new(AdminGetLinguaUsageRequest {
            window: Some(crate::proto::LinguaWindow {
                from_day: "2026-09-01".into(),
                to_day: "2026-09-30".into(),
            }),
        });
        req.extensions_mut().insert(AuthIdentity {
            user_id: "admin-1".into(),
            audience: "back-office".into(),
            roles: scopes
                .iter()
                .flat_map(|(_, rs)| rs.iter().map(|r| r.to_string()))
                .collect(),
            roles_by_scope: by_scope,
        });
        req
    }

    #[tokio::test]
    async fn lingua_admin_and_break_glass_are_accepted() {
        assert!(
            grpc()
                .admin_get_lingua_usage(scoped(&[("lingua", &["admin"])]))
                .await
                .is_ok()
        );
        assert!(
            grpc()
                .admin_get_lingua_usage(scoped(&[("global", &["admin"])]))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn an_admin_of_another_product_is_refused() {
        let err = grpc()
            .admin_get_lingua_usage(scoped(&[("music", &["admin"])]))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn a_flat_legacy_token_is_refused() {
        // Roles present but no scoped roles at all (a pre-scope console token).
        let err = grpc()
            .admin_get_lingua_usage(scoped(&[]))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn a_missing_identity_is_unauthenticated() {
        let req = Request::new(AdminGetLinguaUsageRequest { window: None });
        let err = grpc().admin_get_lingua_usage(req).await.unwrap_err();
        assert_eq!(err.code(), Code::Unauthenticated);
    }

    #[test]
    fn every_series_metric_maps_from_the_wire() {
        assert_eq!(metric_from_proto(0), SeriesMetric::WordsLearned);
        assert_eq!(metric_from_proto(1), SeriesMetric::Reviews);
        assert_eq!(metric_from_proto(2), SeriesMetric::Exposures);
        assert_eq!(metric_from_proto(3), SeriesMetric::UnknownSeen);
        assert_eq!(metric_from_proto(99), SeriesMetric::WordsLearned);
    }
}
