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
use crate::known_words::{KnownWordsModule, StatusChange, StatusOpInput};
use crate::proto::known_words_service_server::KnownWordsService;
use crate::proto::{
    GetSnapshotRequest, GetSnapshotResponse, PullChangesRequest, PullChangesResponse,
    PushOpsRequest, PushOpsResponse, StatusChange as ProtoChange,
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

#[tonic::async_trait]
impl KnownWordsService for KnownWordsGrpc {
    async fn push_ops(
        &self,
        req: Request<PushOpsRequest>,
    ) -> Result<Response<PushOpsResponse>, Status> {
        let user = caller(&req)?;
        let ops = req
            .into_inner()
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
        let (applied, cursor) = self.module.push_ops(&user, ops, now_ms()).await?;
        Ok(Response::new(PushOpsResponse { applied, cursor }))
    }

    async fn pull_changes(
        &self,
        req: Request<PullChangesRequest>,
    ) -> Result<Response<PullChangesResponse>, Status> {
        let user = caller(&req)?;
        let (changes, cursor) = self
            .module
            .pull_changes(&user, req.into_inner().cursor)
            .await?;
        Ok(Response::new(PullChangesResponse {
            changes: changes.into_iter().map(to_proto).collect(),
            cursor,
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
        Ok(Response::new(GetSnapshotResponse {
            unchanged,
            etag,
            statuses: statuses.into_iter().map(to_proto).collect(),
        }))
    }
}
