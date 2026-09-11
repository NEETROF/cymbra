// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Cymbra Lingua backend module (change: add-lingua-backend). Owns the `lingua`
//! Postgres schema and the `cymbra.lingua.v1` sync services — statuses
//! ([`KnownWordsService`]), whole cards ([`DeckService`]) and daily learning aggregates
//! ([`StatsService`]) — modelled on `backend/music`. It is a MODULE, not a split: the
//! boundary next door is a Rust trait (the injected [`cymbra_user_port::UserPort`]),
//! never a gRPC client. Deployable inert: without `CYMBRA_LINGUA_DATABASE_URL` the
//! composition root wires none of it.
//!
//! Privacy allow-list (spec `lingua-sync`): only lemma statuses, user-created cards and
//! day-grained aggregates ever cross the wire — never a browsing URL, page text, or
//! reading history.

pub mod grpc_util;

pub mod known_words;
pub mod known_words_core;
pub mod known_words_grpc;
pub mod pg_known_words;

pub mod deck;
pub mod deck_grpc;
pub mod pg_deck;

pub mod pg_stats;
pub mod stats;
pub mod stats_core;
pub mod stats_grpc;

pub use deck::{Card, DeckModule, DeckRepo};
pub use deck_grpc::DeckGrpc;
pub use known_words::{KnownWordsModule, KnownWordsRepo, Status, StatusChange};
pub use known_words_grpc::KnownWordsGrpc;
pub use pg_deck::PgDeckRepo;
pub use pg_known_words::PgKnownWordsRepo;
pub use pg_stats::PgStatsRepo;
pub use stats::{DailyStat, StatsModule, StatsRepo};
pub use stats_grpc::StatsGrpc;

/// The generated `cymbra.lingua.v1` server stubs + messages.
#[allow(clippy::result_large_err)]
pub mod proto {
    tonic::include_proto!("cymbra.lingua.v1");
}

/// The module's Postgres schema.
pub const SCHEMA: &str = "lingua";

/// Embedded migrations for the `lingua` schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
