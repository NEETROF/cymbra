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
use crate::grpc_util::identity;
use crate::pack_registry::DataPack;
use crate::proto::lingua_admin_service_server::LinguaAdminService;
use crate::proto::{
    AdminGetLinguaUsageRequest, AdminGetLinguaUsageResponse, AdminGetLinguaUsageSeriesRequest,
    AdminGetLinguaUsageSeriesResponse, AdminListDataPacksRequest, AdminListDataPacksResponse,
    LinguaDataPack, LinguaLanguageUsage, LinguaSeriesPoint,
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
        _ => SeriesMetric::WordsLearned,
    }
}

/// Pull the (from_day, to_day) out of a required window, erroring if it is absent.
fn window(w: Option<crate::proto::LinguaWindow>) -> Result<(String, String), Status> {
    let w = w.ok_or_else(|| Status::invalid_argument("window is required"))?;
    Ok((w.from_day, w.to_day))
}

fn pack_to_proto(p: &DataPack) -> LinguaDataPack {
    LinguaDataPack {
        studied: p.studied.clone(),
        native: p.native.clone(),
        pack_version: p.pack_version.clone(),
        analyzer_version: p.analyzer_version.clone(),
        built_at: p.built_at.clone(),
        size_bytes: p.size_bytes,
        notice: p.notice.clone(),
    }
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
            by_language: u
                .by_language
                .into_iter()
                .map(|l| LinguaLanguageUsage {
                    language: l.language,
                    active_accounts: l.active_accounts,
                    words_learned: l.words_learned,
                    reviews: l.reviews,
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

    async fn admin_list_data_packs(
        &self,
        req: Request<AdminListDataPacksRequest>,
    ) -> Result<Response<AdminListDataPacksResponse>, Status> {
        require_admin_in_scope(&identity(&req)?, LINGUA_SCOPE)?;
        Ok(Response::new(AdminListDataPacksResponse {
            packs: self.module.list_packs().iter().map(pack_to_proto).collect(),
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
    }

    fn grpc() -> LinguaAdminGrpc {
        LinguaAdminGrpc::new(Arc::new(
            LinguaAdminModule::new(Arc::new(QuietRepo)).expect("manifest parses"),
        ))
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
}
