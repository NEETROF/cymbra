// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Postgres adapter for [`KnownWordsRepo`]. Thin sqlx glue — coverage-excluded; the
//! LWW/clamp logic is host-tested in `known_words_core` + `known_words`. The
//! conflict-update WHERE applies the same last-write-wins rule in SQL, and every write
//! stamps a fresh `nextval('lingua.change_seq')` so the cursor pull sees it.

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::known_words::{
    DeclaredLevelChange, DeclaredLevelOpInput, KnownWordsRepo, StatusChange, StatusOpInput,
};

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("lingua db: {e}"))
}

fn uid(user: &str) -> Result<Uuid> {
    Uuid::parse_str(user).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}

pub struct PgKnownWordsRepo {
    pool: PgPool,
}

impl PgKnownWordsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KnownWordsRepo for PgKnownWordsRepo {
    async fn apply_op(&self, user: &str, op: &StatusOpInput) -> Result<bool> {
        let affected = sqlx::query(
            "INSERT INTO lingua.word_statuses \
               (user_id, language, lemma, status, provenance, updated_at, device_id, seq) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, nextval('lingua.change_seq')) \
             ON CONFLICT (user_id, language, lemma) DO UPDATE SET \
               status = excluded.status, provenance = excluded.provenance, \
               updated_at = excluded.updated_at, device_id = excluded.device_id, \
               seq = nextval('lingua.change_seq') \
             WHERE excluded.updated_at > lingua.word_statuses.updated_at \
                OR (excluded.updated_at = lingua.word_statuses.updated_at \
                    AND excluded.device_id > lingua.word_statuses.device_id)",
        )
        .bind(uid(user)?)
        .bind(&op.language)
        .bind(&op.lemma)
        .bind(&op.status)
        .bind(&op.provenance)
        .bind(op.client_ts)
        .bind(&op.device_id)
        .execute(&self.pool)
        .await
        .map_err(internal)?
        .rows_affected();
        Ok(affected > 0)
    }

    async fn tip_cursor(&self, user: &str) -> Result<i64> {
        // Statuses and declared levels share `lingua.change_seq`, so the cursor is
        // the max sequence over both tables — a change to either advances it.
        let row = sqlx::query(
            "SELECT GREATEST( \
               (SELECT COALESCE(MAX(seq), 0) FROM lingua.word_statuses WHERE user_id = $1), \
               (SELECT COALESCE(MAX(seq), 0) FROM lingua.declared_levels WHERE user_id = $1) \
             ) AS c",
        )
        .bind(uid(user)?)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.get::<i64, _>("c"))
    }

    async fn changes_since(&self, user: &str, cursor: i64) -> Result<Vec<StatusChange>> {
        let rows = sqlx::query(
            "SELECT language, lemma, status, updated_at, seq FROM lingua.word_statuses \
             WHERE user_id = $1 AND seq > $2 ORDER BY seq",
        )
        .bind(uid(user)?)
        .bind(cursor)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| StatusChange {
                language: r.get("language"),
                lemma: r.get("lemma"),
                status: r.get("status"),
                updated_at: r.get("updated_at"),
                sequence: r.get("seq"),
            })
            .collect())
    }

    async fn snapshot(&self, user: &str) -> Result<Vec<StatusChange>> {
        self.changes_since(user, 0).await
    }

    async fn apply_level_op(&self, user: &str, op: &DeclaredLevelOpInput) -> Result<bool> {
        let affected = sqlx::query(
            "INSERT INTO lingua.declared_levels \
               (user_id, language, level, updated_at, device_id, seq) \
             VALUES ($1, $2, $3, $4, $5, nextval('lingua.change_seq')) \
             ON CONFLICT (user_id, language) DO UPDATE SET \
               level = excluded.level, updated_at = excluded.updated_at, \
               device_id = excluded.device_id, seq = nextval('lingua.change_seq') \
             WHERE excluded.updated_at > lingua.declared_levels.updated_at \
                OR (excluded.updated_at = lingua.declared_levels.updated_at \
                    AND excluded.device_id > lingua.declared_levels.device_id)",
        )
        .bind(uid(user)?)
        .bind(&op.language)
        .bind(&op.level)
        .bind(op.client_ts)
        .bind(&op.device_id)
        .execute(&self.pool)
        .await
        .map_err(internal)?
        .rows_affected();
        Ok(affected > 0)
    }

    async fn level_changes_since(
        &self,
        user: &str,
        cursor: i64,
    ) -> Result<Vec<DeclaredLevelChange>> {
        let rows = sqlx::query(
            "SELECT language, level, updated_at, seq FROM lingua.declared_levels \
             WHERE user_id = $1 AND seq > $2 ORDER BY seq",
        )
        .bind(uid(user)?)
        .bind(cursor)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .iter()
            .map(|r| DeclaredLevelChange {
                language: r.get("language"),
                level: r.get("level"),
                updated_at: r.get("updated_at"),
                sequence: r.get("seq"),
            })
            .collect())
    }

    async fn level_snapshot(&self, user: &str) -> Result<Vec<DeclaredLevelChange>> {
        self.level_changes_since(user, 0).await
    }
}
