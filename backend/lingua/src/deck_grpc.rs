// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Tonic adapter for DeckService — thin transport (coverage-excluded); logic is
//! host-tested in `deck`. Media contents are never transported, and a card's deprecated
//! `source` is ignored on push and returned empty.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::deck::{Card, DeckModule};
use crate::grpc_util::{caller, now_ms};
use crate::language_core::{gloss_language, normalise};
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
        // Empty from a client that predates the field: read as English.
        language: normalise(&o.language),
        client_id: o.client_id,
        lemma: o.lemma,
        surface_form: o.surface_form,
        source_sentence: o.source_sentence,
        gloss: o.gloss,
        // Empty from a client that predates the field: every gloss it holds is French.
        gloss_language: gloss_language(&o.gloss_language),
        fsrs_state: o.fsrs_state,
        deleted: o.deleted,
        updated_at: o.client_ts,
        device_id: o.device_id,
        sequence: 0,
    }
}

// `source` is deprecated in the contract; it is only ever written empty.
#[allow(deprecated)]
fn to_proto(c: Card) -> CardOp {
    CardOp {
        client_id: c.client_id,
        lemma: c.lemma,
        surface_form: c.surface_form,
        source_sentence: c.source_sentence,
        // The page a card came from stays on the device (add-lingua-privacy-controls).
        source: String::new(),
        gloss: c.gloss,
        fsrs_state: c.fsrs_state,
        deleted: c.deleted,
        client_ts: c.updated_at,
        device_id: c.device_id,
        language: c.language,
        gloss_language: c.gloss_language,
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
        let r = req.into_inner();
        // An empty list is a client that predates card languages: English only; an unset
        // flag, one that predates gloss labels: French-glossed cards only.
        let (cards, cursor) = self
            .module
            .pull_cards(&user, r.cursor, &r.languages, r.any_gloss_language)
            .await?;
        Ok(Response::new(PullCardsResponse {
            cards: cards.into_iter().map(to_proto).collect(),
            cursor,
        }))
    }
}

// The adapter is excluded from the coverage gate; these tests are for its correctness:
// the edge's defaults and what it maps onto the module.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MockErasureMarks;
    use crate::deck::MockDeckRepo;
    use cymbra_platform::AuthIdentity;

    fn as_reader<T>(body: T) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut().insert(AuthIdentity {
            user_id: "u1".into(),
            ..AuthIdentity::default()
        });
        req
    }

    // --- A card carries the language of its gloss (add-lingua-native-language-server) ---

    #[test]
    fn an_op_without_a_gloss_language_reads_fr_at_the_edge() {
        let card = from_proto(CardOp::default());
        assert_eq!(card.gloss_language, "fr");
        assert_eq!(
            card.language, "en",
            "the studied language keeps its own default"
        );
    }

    #[test]
    fn an_upper_case_gloss_language_is_normalised_at_the_edge() {
        let op = CardOp {
            gloss_language: "EN".into(),
            ..CardOp::default()
        };
        assert_eq!(from_proto(op).gloss_language, "en");
    }

    #[test]
    fn a_pulled_card_carries_its_gloss_language_as_stored() {
        let stored = Card {
            gloss_language: "tlh".into(),
            ..Card::default()
        };
        assert_eq!(to_proto(stored).gloss_language, "tlh");
    }

    #[tokio::test]
    async fn a_pull_maps_any_gloss_language_onto_the_module() {
        for flag in [false, true] {
            let mut repo = MockDeckRepo::new();
            repo.expect_changes_since()
                .withf(move |_, cursor, languages, any_gloss_language| {
                    *cursor == 7 && languages == ["en"] && *any_gloss_language == flag
                })
                .times(1)
                .returning(|_, _, _, _| Ok(vec![]));
            let module = DeckModule::new(Arc::new(repo), Arc::new(MockErasureMarks::new()));
            let grpc = DeckGrpc::new(Arc::new(module));
            let out = grpc
                .pull_cards(as_reader(PullCardsRequest {
                    cursor: 7,
                    languages: vec![],
                    any_gloss_language: flag,
                }))
                .await
                .unwrap()
                .into_inner();
            assert_eq!(out.cursor, 7, "nothing returned, the cursor stays");
        }
    }
}
