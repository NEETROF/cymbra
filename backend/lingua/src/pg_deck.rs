// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Postgres adapter for [`DeckRepo`]. Thin sqlx glue — coverage-excluded; LWW logic is
//! host-tested in `deck`. Media contents are never stored (allow-list): no media column.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::deck::{Card, DeckRepo};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("lingua db: {e}"))
}

fn uid(user: &str) -> Result<Uuid> {
    Uuid::parse_str(user).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}

pub struct PgDeckRepo {
    pool: PgPool,
}

impl PgDeckRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeckRepo for PgDeckRepo {
    async fn apply_card(&self, user: &str, card: &Card) -> Result<bool> {
        let affected = sqlx::query(
            "INSERT INTO lingua.cards \
               (user_id, client_id, lemma, surface_form, source_sentence, source, gloss, \
                fsrs_state, deleted, updated_at, device_id, seq) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, nextval('lingua.change_seq')) \
             ON CONFLICT (user_id, client_id) DO UPDATE SET \
               lemma = excluded.lemma, surface_form = excluded.surface_form, \
               source_sentence = excluded.source_sentence, source = excluded.source, \
               gloss = excluded.gloss, fsrs_state = excluded.fsrs_state, \
               deleted = excluded.deleted, updated_at = excluded.updated_at, \
               device_id = excluded.device_id, seq = nextval('lingua.change_seq') \
             WHERE excluded.updated_at > lingua.cards.updated_at \
                OR (excluded.updated_at = lingua.cards.updated_at \
                    AND excluded.device_id > lingua.cards.device_id)",
        )
        .bind(uid(user)?)
        .bind(&card.client_id)
        .bind(&card.lemma)
        .bind(&card.surface_form)
        .bind(&card.source_sentence)
        .bind(&card.source)
        .bind(&card.gloss)
        .bind(&card.fsrs_state)
        .bind(card.deleted)
        .bind(card.updated_at)
        .bind(&card.device_id)
        .execute(&self.pool)
        .await
        .map_err(internal)?
        .rows_affected();
        Ok(affected > 0)
    }

    async fn tip_cursor(&self, user: &str) -> Result<i64> {
        let row =
            sqlx::query("SELECT COALESCE(MAX(seq), 0) AS c FROM lingua.cards WHERE user_id = $1")
                .bind(uid(user)?)
                .fetch_one(&self.pool)
                .await
                .map_err(internal)?;
        Ok(row.get::<i64, _>("c"))
    }

    async fn changes_since(&self, user: &str, cursor: i64) -> Result<Vec<Card>> {
        let rows = sqlx::query(
            "SELECT client_id, lemma, surface_form, source_sentence, source, gloss, fsrs_state, \
                    deleted, updated_at, device_id, seq \
             FROM lingua.cards WHERE user_id = $1 AND seq > $2 ORDER BY seq",
        )
        .bind(uid(user)?)
        .bind(cursor)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| Card {
                client_id: r.get("client_id"),
                lemma: r.get("lemma"),
                surface_form: r.get("surface_form"),
                source_sentence: r.get("source_sentence"),
                source: r.get("source"),
                gloss: r.get::<Option<String>, _>("gloss").unwrap_or_default(),
                fsrs_state: r.get("fsrs_state"),
                deleted: r.get("deleted"),
                updated_at: r.get("updated_at"),
                device_id: r.get("device_id"),
                sequence: r.get("seq"),
            })
            .collect())
    }
}
