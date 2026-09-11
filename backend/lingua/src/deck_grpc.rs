// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Tonic adapter for DeckService — thin transport (coverage-excluded); logic is
//! host-tested in `deck`. Media contents are never transported.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::deck::{Card, DeckModule};
use crate::grpc_util::{caller, now_ms};
use crate::proto::deck_service_server::DeckService;
use crate::proto::{
    CardOp, PullCardsRequest, PullCardsResponse, PushCardsRequest, PushCardsResponse,
};

pub struct DeckGrpc {
    module: Arc<DeckModule>,
}

impl DeckGrpc {
    pub fn new(module: Arc<DeckModule>) -> Self {
        Self { module }
    }
}

fn from_proto(o: CardOp) -> Card {
    Card {
        client_id: o.client_id,
        lemma: o.lemma,
        surface_form: o.surface_form,
        source_sentence: o.source_sentence,
        source: o.source,
        gloss: o.gloss,
        fsrs_state: o.fsrs_state,
        deleted: o.deleted,
        updated_at: o.client_ts,
        device_id: o.device_id,
        sequence: 0,
    }
}

fn to_proto(c: Card) -> CardOp {
    CardOp {
        client_id: c.client_id,
        lemma: c.lemma,
        surface_form: c.surface_form,
        source_sentence: c.source_sentence,
        source: c.source,
        gloss: c.gloss,
        fsrs_state: c.fsrs_state,
        deleted: c.deleted,
        client_ts: c.updated_at,
        device_id: c.device_id,
    }
}

#[tonic::async_trait]
impl DeckService for DeckGrpc {
    async fn push_cards(
        &self,
        req: Request<PushCardsRequest>,
    ) -> Result<Response<PushCardsResponse>, Status> {
        let user = caller(&req)?;
        let cards = req.into_inner().cards.into_iter().map(from_proto).collect();
        let (applied, cursor) = self.module.push_cards(&user, cards, now_ms()).await?;
        Ok(Response::new(PushCardsResponse { applied, cursor }))
    }

    async fn pull_cards(
        &self,
        req: Request<PullCardsRequest>,
    ) -> Result<Response<PullCardsResponse>, Status> {
        let user = caller(&req)?;
        let (cards, cursor) = self
            .module
            .pull_cards(&user, req.into_inner().cursor)
            .await?;
        Ok(Response::new(PullCardsResponse {
            cards: cards.into_iter().map(to_proto).collect(),
            cursor,
        }))
    }
}
