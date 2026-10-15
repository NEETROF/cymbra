// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! gRPC adapter for [`UsageCountsService`] (change: add-lingua-usage-counts). Thin: it
//! reads the message fields and nothing else, and resolves the client address
//! (`cymbra_platform::client_addr`) for the flood guard alone. Mounted WITHOUT the auth
//! interceptor, so an `Authorization` header is never even read: a report cannot resolve
//! an account. Coverage-excluded like the other `*_grpc.rs` adapters; every decision is in
//! [`UsageCountsModule`](crate::usage_counts::UsageCountsModule).

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::usage_counts_service_server::UsageCountsService;
use crate::proto::{ReportRequest, ReportResponse};
use crate::usage_counts::{Report, UsageCountsModule};

pub struct UsageCountsGrpc {
    module: Arc<UsageCountsModule>,
}

impl UsageCountsGrpc {
    pub fn new(module: Arc<UsageCountsModule>) -> Self {
        Self { module }
    }
}

/// The address the flood guard keys, resolved like every rate limit: Caddy's first
/// `X-Forwarded-For` hop, else `X-Real-IP`, else the peer.
fn client_addr<T>(req: &Request<T>) -> String {
    cymbra_platform::client_addr::resolve(
        |name| {
            req.metadata()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        },
        req.remote_addr().map(|a| a.ip()),
    )
}

#[tonic::async_trait]
impl UsageCountsService for UsageCountsGrpc {
    async fn report(
        &self,
        req: Request<ReportRequest>,
    ) -> Result<Response<ReportResponse>, Status> {
        let addr = client_addr(&req);
        let r = req.into_inner();
        self.module
            .report(
                &addr,
                &Report {
                    install_day: r.install_day,
                    browser: r.browser,
                    version: r.version,
                    steps: r.steps,
                },
            )
            .await?;
        Ok(Response::new(ReportResponse {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage_counts::{FloodGuard, MockClock, MockUsageCountsPolicy, MockUsageCountsRepo};
    use cymbra_platform::AuthIdentity;
    use tonic::Code;
    use tonic::metadata::MetadataValue;

    const NOW: u64 = 20_747 * 86_400;

    fn grpc(repo: MockUsageCountsRepo) -> UsageCountsGrpc {
        let mut policy = MockUsageCountsPolicy::new();
        policy.expect_enabled().return_const(true);
        policy.expect_ceiling().return_const(5_000i64);
        policy
            .expect_latest_version()
            .return_const("1.10".to_string());
        let mut clock = MockClock::new();
        clock.expect_now_secs().return_const(NOW);
        UsageCountsGrpc::new(Arc::new(UsageCountsModule::new(
            Arc::new(repo),
            Arc::new(policy),
            Arc::new(clock),
            FloodGuard::with_key([0; 32], 1, 10),
        )))
    }

    fn request(bearer: bool, forwarded: &str) -> Request<ReportRequest> {
        let mut req = Request::new(ReportRequest {
            install_day: 20_747,
            browser: 1,
            version: "1.10".into(),
            steps: vec![1],
        });
        if bearer {
            req.metadata_mut().insert(
                "authorization",
                MetadataValue::from_static("Bearer eyJ.a.signed.in.reader"),
            );
        }
        req.metadata_mut()
            .insert("x-forwarded-for", forwarded.parse().unwrap());
        req
    }

    #[tokio::test]
    async fn a_request_carrying_a_bearer_resolves_no_identity() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let req = request(true, "203.0.113.5");
        // No interceptor ran: nothing put an identity on the request, and the service
        // does not need one.
        assert!(req.extensions().get::<AuthIdentity>().is_none());
        grpc(repo).report(req).await.unwrap();
    }

    #[tokio::test]
    async fn the_forwarded_address_feeds_the_guard_only() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .times(2)
            .returning(|_, _, _, _, _| Ok(()));
        let svc = grpc(repo);
        svc.report(request(false, "203.0.113.5")).await.unwrap();
        // The guard here admits one request per address: the same forwarded address is
        // refused, another one is not.
        let refused = svc
            .report(request(false, "203.0.113.5, 10.0.0.1"))
            .await
            .unwrap_err();
        assert_eq!(refused.code(), Code::ResourceExhausted);
        svc.report(request(false, "198.51.100.7")).await.unwrap();
    }
}
