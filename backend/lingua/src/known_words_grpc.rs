// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Tonic adapter for KnownWordsService — thin transport (coverage-excluded); the logic
//! is host-tested in `known_words` / `known_words_core`. The caller is the token's user
//! (from the interceptor), never the request body.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::grpc_util::{caller, now_ms};
use crate::known_words::{
    DeclaredLevelChange, DeclaredLevelOpInput, KnownWordsModule, StatusChange, StatusOpInput,
};
use crate::proto::known_words_service_server::KnownWordsService;
use crate::proto::{
    DeclaredLevelChange as ProtoLevelChange, GetSnapshotRequest, GetSnapshotResponse,
    PullChangesRequest, PullChangesResponse, PushOpsRequest, PushOpsResponse,
    StatusChange as ProtoChange,
};

pub struct KnownWordsGrpc {
    module: Arc<KnownWordsModule>,
}

impl KnownWordsGrpc {
    pub fn new(module: Arc<KnownWordsModule>) -> Self {
        Self { module }
    }
}

fn to_proto(c: StatusChange) -> ProtoChange {
    ProtoChange {
        language: c.language,
        lemma: c.lemma,
        status: c.status,
        updated_at: c.updated_at,
        sequence: c.sequence,
    }
}

fn level_to_proto(c: DeclaredLevelChange) -> ProtoLevelChange {
    ProtoLevelChange {
        language: c.language,
        level: c.level,
        updated_at: c.updated_at,
        sequence: c.sequence,
    }
}

#[tonic::async_trait]
impl KnownWordsService for KnownWordsGrpc {
    async fn push_ops(
        &self,
        req: Request<PushOpsRequest>,
    ) -> Result<Response<PushOpsResponse>, Status> {
        let user = caller(&req)?;
        let body = req.into_inner();
        let ops = body
            .ops
            .into_iter()
            .map(|o| StatusOpInput {
                language: o.language,
                lemma: o.lemma,
                status: o.status,
                provenance: o.provenance,
                client_ts: o.client_ts,
                device_id: o.device_id,
            })
            .collect();
        let levels = body
            .declared_levels
            .into_iter()
            .map(|o| DeclaredLevelOpInput {
                language: o.language,
                level: o.level,
                client_ts: o.client_ts,
                device_id: o.device_id,
            })
            .collect();
        // One `now` for both drains, so a status and a level pushed together clamp
        // against the same clock. Levels share the cursor, so the second call's
        // tip is the authoritative one to return.
        let now = now_ms();
        let (status_applied, _) = self.module.push_ops(&user, ops, now).await?;
        let (level_applied, cursor) = self.module.push_level_ops(&user, levels, now).await?;
        Ok(Response::new(PushOpsResponse {
            applied: status_applied + level_applied,
            cursor,
        }))
    }

    async fn pull_changes(
        &self,
        req: Request<PullChangesRequest>,
    ) -> Result<Response<PullChangesResponse>, Status> {
        let user = caller(&req)?;
        let cursor = req.into_inner().cursor;
        let (changes, status_next) = self.module.pull_changes(&user, cursor).await?;
        let (levels, level_next) = self.module.pull_level_changes(&user, cursor).await?;
        Ok(Response::new(PullChangesResponse {
            changes: changes.into_iter().map(to_proto).collect(),
            cursor: status_next.max(level_next),
            declared_levels: levels.into_iter().map(level_to_proto).collect(),
        }))
    }

    async fn get_snapshot(
        &self,
        req: Request<GetSnapshotRequest>,
    ) -> Result<Response<GetSnapshotResponse>, Status> {
        let user = caller(&req)?;
        let (unchanged, etag, statuses) = self
            .module
            .get_snapshot(&user, &req.into_inner().etag)
            .await?;
        // When the ETag matched, the whole snapshot is unchanged — levels included —
        // so skip the level read too.
        let levels = if unchanged {
            Vec::new()
        } else {
            self.module.level_snapshot(&user).await?
        };
        Ok(Response::new(GetSnapshotResponse {
            unchanged,
            etag,
            statuses: statuses.into_iter().map(to_proto).collect(),
            declared_levels: levels.into_iter().map(level_to_proto).collect(),
        }))
    }
}
