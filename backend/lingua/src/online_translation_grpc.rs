// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! gRPC adapter for [`OnlineTranslationService`] (change: add-lingua-online-translation).
//! Thin: the caller from the interceptor only, proto mapping; every decision is in
//! [`OnlineTranslationModule`](crate::online_translation::OnlineTranslationModule).
//! Coverage-excluded like the other `*_grpc.rs` adapters.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::grpc_util::identity;
use crate::online_translation::{self as module, OnlineTranslationModule, Outcome};
use crate::proto::online_translation_service_server::OnlineTranslationService;
use crate::proto::translate_response::Outcome as WireOutcome;
use crate::proto::{
    LimitReached, TranslateRequest, TranslateResponse, Translated, TranslationMark, Unavailable,
};

pub struct OnlineTranslationGrpc {
    module: Arc<OnlineTranslationModule>,
}

impl OnlineTranslationGrpc {
    pub fn new(module: Arc<OnlineTranslationModule>) -> Self {
        Self { module }
    }
}

#[tonic::async_trait]
impl OnlineTranslationService for OnlineTranslationGrpc {
    async fn translate(
        &self,
        req: Request<TranslateRequest>,
    ) -> Result<Response<TranslateResponse>, Status> {
        let id = identity(&req)?;
        let r = req.into_inner();
        let outcome = self
            .module
            .translate(
                &id,
                &module::Request {
                    sentence: r.sentence,
                    selection_start: r.selection_start,
                    selection_end: r.selection_end,
                    source_language: r.source_language,
                    target_language: r.target_language,
                },
            )
            .await?;
        let outcome = match outcome {
            Outcome::Translated { translation, marks } => WireOutcome::Translated(Translated {
                translation,
                marks: marks
                    .into_iter()
                    .map(|(start, end)| TranslationMark { start, end })
                    .collect(),
            }),
            Outcome::Unavailable => WireOutcome::Unavailable(Unavailable {}),
            Outcome::LimitReached { resets_at } => {
                WireOutcome::LimitReached(LimitReached { resets_at })
            }
        };
        Ok(Response::new(TranslateResponse {
            outcome: Some(outcome),
        }))
    }
}
