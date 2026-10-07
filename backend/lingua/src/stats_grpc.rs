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
use crate::language_core::{native_language, normalise};
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
        language: normalise(&s.language),
        device_id: s.device_id,
        exposures: s.exposures,
        words_learned: s.words_learned,
        reviews_done: s.reviews_done,
        unknown_seen: s.unknown_seen,
        // Empty from a client that predates the field: its reader is French-native.
        native_language: native_language(&s.native_language),
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
        // Empty still means every language; a named one is normalised like on the way in.
        let language = if r.language.trim().is_empty() {
            None
        } else {
            Some(normalise(&r.language))
        };
        let stats = self
            .module
            .get_stats(&user, r.from_day, r.to_day, language.as_deref())
            .await?;
        Ok(Response::new(GetStatsResponse {
            stats: stats.into_iter().map(to_proto).collect(),
        }))
    }
}

// The adapter is excluded from the coverage gate; these tests are for its correctness:
// the edge's defaults.
#[cfg(test)]
mod tests {
    use super::*;

    // --- A daily statistic carries the native language of its device
    // (add-lingua-native-language-server) ---

    #[test]
    fn a_stat_without_a_native_language_reads_fr_at_the_edge() {
        let stat = from_proto(ProtoDaily::default());
        assert_eq!(stat.native_language, "fr");
        assert_eq!(
            stat.language, "en",
            "the studied language keeps its own default"
        );
        assert_eq!(
            stat.unknown_seen, None,
            "presence still marks an up-to-date client"
        );
    }

    #[test]
    fn a_regional_native_language_is_normalised_at_the_edge() {
        let stat = from_proto(ProtoDaily {
            native_language: "es-MX".into(),
            unknown_seen: Some(1),
            ..ProtoDaily::default()
        });
        assert_eq!(stat.native_language, "es");
        assert_eq!(stat.unknown_seen, Some(1));
    }
}
