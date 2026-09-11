// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Tonic adapter for StatsService — thin transport (coverage-excluded); the
//! consolidation is host-tested in `stats_core`. Reads are scoped to the caller.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::grpc_util::caller;
use crate::proto::stats_service_server::StatsService;
use crate::proto::{
    ConsolidatedStat as ProtoConsolidated, DailyStat as ProtoDaily, GetStatsRequest,
    GetStatsResponse, UpsertDailyStatsRequest, UpsertDailyStatsResponse,
};
use crate::stats::{ConsolidatedStat, DailyStat, StatsModule};

pub struct StatsGrpc {
    module: Arc<StatsModule>,
}

impl StatsGrpc {
    pub fn new(module: Arc<StatsModule>) -> Self {
        Self { module }
    }
}

fn from_proto(s: ProtoDaily) -> DailyStat {
    DailyStat {
        day: s.day,
        language: s.language,
        device_id: s.device_id,
        exposures: s.exposures,
        words_learned: s.words_learned,
        reviews_done: s.reviews_done,
    }
}

fn to_proto(c: ConsolidatedStat) -> ProtoConsolidated {
    ProtoConsolidated {
        day: c.day,
        language: c.language,
        exposures: c.exposures,
        words_learned: c.words_learned,
        reviews_done: c.reviews_done,
    }
}

#[tonic::async_trait]
impl StatsService for StatsGrpc {
    async fn upsert_daily_stats(
        &self,
        req: Request<UpsertDailyStatsRequest>,
    ) -> Result<Response<UpsertDailyStatsResponse>, Status> {
        let user = caller(&req)?;
        let stats = req.into_inner().stats.into_iter().map(from_proto).collect();
        let upserted = self.module.upsert_stats(&user, stats).await?;
        Ok(Response::new(UpsertDailyStatsResponse { upserted }))
    }

    async fn get_stats(
        &self,
        req: Request<GetStatsRequest>,
    ) -> Result<Response<GetStatsResponse>, Status> {
        let user = caller(&req)?;
        let r = req.into_inner();
        let language = if r.language.is_empty() {
            None
        } else {
            Some(r.language.as_str())
        };
        let stats = self
            .module
            .get_stats(&user, r.from_day, r.to_day, language)
            .await?;
        Ok(Response::new(GetStatsResponse {
            stats: stats.into_iter().map(to_proto).collect(),
        }))
    }
}
