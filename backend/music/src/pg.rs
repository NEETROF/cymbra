//! Postgres-backed [`CatalogRepo`] — thin I/O glue (excluded from the coverage
//! gate; exercised by integration tests against a real database).
//!
//! Uses the runtime `sqlx::query(...).bind(...)` API (not the compile-time
//! `query!` macros), matching `backend/user/src/pg.rs`. Table names are fully
//! qualified (`music.catalog_scores`) so it works regardless of the connecting
//! role's `search_path`.

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Row, postgres::PgArguments, postgres::PgRow, query::Query};

use crate::catalog_search::{CatalogHit, CatalogSearchParams, CatalogSearchRepo};
use crate::repo::{CatalogEntry, CatalogRepo, ScoreFacets, ScoreMeta};

/// The shared [`ScoreMeta`] columns (descriptive + facets), in the canonical bind
/// order of [`bind_meta`] / [`meta_from_row`]. Both `catalog_scores` and
/// `user_scores` expose these column names, so their INSERT/SELECT statements
/// share this one fragment instead of repeating 18 columns each.
pub(crate) const META_COLS: &str = "title, composer, title_norm, work_key, key_fifths, \
     time_sig, measure_count, is_piano, min_note_value, has_tuplets, has_dotted, has_chords, \
     lowest_midi, highest_midi, staff_count, note_count, tempo_bpm, has_dynamics";

/// Append the 18 [`ScoreMeta`] binds to `q`, in [`META_COLS`] order. The caller
/// places the matching `$n..$n+17` placeholders as a contiguous trailing block.
pub(crate) fn bind_meta<'q>(
    q: Query<'q, Postgres, PgArguments>,
    m: &'q ScoreMeta,
) -> Query<'q, Postgres, PgArguments> {
    q.bind(&m.title)
        .bind(&m.composer)
        .bind(&m.title_norm)
        .bind(&m.work_key)
        .bind(m.key_fifths)
        .bind(&m.time_sig)
        .bind(m.measure_count)
        .bind(m.is_piano)
        .bind(m.facets.min_note_value.map(i16::from))
        .bind(m.facets.has_tuplets)
        .bind(m.facets.has_dotted)
        .bind(m.facets.has_chords)
        .bind(m.facets.lowest_midi.map(i16::from))
        .bind(m.facets.highest_midi.map(i16::from))
        .bind(i16::from(m.facets.staff_count))
        .bind(m.facets.note_count as i32)
        .bind(m.facets.tempo_bpm.map(i32::from))
        .bind(m.facets.has_dynamics)
}

/// Decode the [`META_COLS`] from a row into a [`ScoreMeta`] (by column name, so
/// the SELECT column order is irrelevant).
pub(crate) fn meta_from_row(r: &PgRow) -> ScoreMeta {
    ScoreMeta {
        title: r.get("title"),
        composer: r.get("composer"),
        title_norm: r.get("title_norm"),
        work_key: r.get("work_key"),
        key_fifths: r.get("key_fifths"),
        time_sig: r.get("time_sig"),
        measure_count: r.get("measure_count"),
        is_piano: r.get("is_piano"),
        facets: ScoreFacets {
            min_note_value: r.get::<Option<i16>, _>("min_note_value").map(|v| v as u8),
            has_tuplets: r.get::<Option<bool>, _>("has_tuplets").unwrap_or(false),
            has_dotted: r.get::<Option<bool>, _>("has_dotted").unwrap_or(false),
            has_chords: r.get::<Option<bool>, _>("has_chords").unwrap_or(false),
            lowest_midi: r.get::<Option<i16>, _>("lowest_midi").map(|v| v as u8),
            highest_midi: r.get::<Option<i16>, _>("highest_midi").map(|v| v as u8),
            staff_count: r.get::<Option<i16>, _>("staff_count").unwrap_or(0) as u8,
            note_count: r.get::<Option<i32>, _>("note_count").unwrap_or(0) as u32,
            tempo_bpm: r.get::<Option<i32>, _>("tempo_bpm").map(|v| v as u16),
            has_dynamics: r.get::<Option<bool>, _>("has_dynamics").unwrap_or(false),
        },
    }
}

/// The catalog INSERT statement. Catalog-specific columns first ($1..$19), then
/// the shared [`ScoreMeta`] block ($20..$37) via [`bind_meta`] — the two adapters
/// share that fragment. `ON CONFLICT (sha256) DO NOTHING` makes re-ingestion
/// idempotent; the affected-row count tells us whether this was a new insert.
///
/// `moderation_status` is written as the literal `'pending'` (never a bind, never
/// from the caller): ingestion must NEVER auto-validate, so the insert path is
/// structurally incapable of persisting any other status (change:
/// add-score-moderation-gating). The DB default guarantees the same, but the
/// explicit literal keeps the invariant local to this statement — and unit-testable
/// (see the test below) without a database.
fn catalog_insert_sql() -> String {
    format!(
        "INSERT INTO music.catalog_scores (\
            id, arranger, source, source_url, source_item_id, license, license_url, \
            confidence, sha256, origin_format, conversion_status, object_key, size_bytes, \
            composer_norm, language, voicing, level, level_source, content_fingerprint, \
            moderation_status, {META_COLS}) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,\
            'pending',$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34,$35,$36,$37) \
         ON CONFLICT (sha256) DO NOTHING"
    )
}

/// Postgres implementation over a pool (typically an admin/ingestion role).
pub struct PgCatalogRepo {
    pool: PgPool,
}

impl PgCatalogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatalogRepo for PgCatalogRepo {
    async fn sha_exists(&self, sha256: &str) -> Result<bool> {
        let row =
            sqlx::query("SELECT EXISTS (SELECT 1 FROM music.catalog_scores WHERE sha256 = $1)")
                .bind(sha256)
                .fetch_one(&self.pool)
                .await
                .context("catalog sha_exists")?;
        Ok(row.get::<bool, _>(0))
    }

    async fn fingerprint_exists(&self, fingerprint: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT EXISTS (SELECT 1 FROM music.catalog_scores WHERE content_fingerprint = $1)",
        )
        .bind(fingerprint)
        .fetch_one(&self.pool)
        .await
        .context("catalog fingerprint_exists")?;
        Ok(row.get::<bool, _>(0))
    }

    async fn insert(&self, e: &CatalogEntry) -> Result<bool> {
        let id = uuid::Uuid::parse_str(&e.id).unwrap_or_else(|_| uuid::Uuid::now_v7());
        let sql = catalog_insert_sql();
        let q = sqlx::query(&sql)
            .bind(id)
            .bind(&e.arranger)
            .bind(&e.source)
            .bind(&e.source_url)
            .bind(&e.source_item_id)
            .bind(&e.license)
            .bind(&e.license_url)
            .bind(&e.confidence)
            .bind(&e.sha256)
            .bind(&e.origin_format)
            .bind(&e.conversion_status)
            .bind(&e.object_key)
            .bind(e.size_bytes)
            .bind(&e.composer_norm)
            .bind(&e.language)
            .bind(&e.voicing)
            .bind(&e.level)
            .bind(&e.level_source)
            .bind(&e.content_fingerprint);
        let result = bind_meta(q, &e.meta)
            .execute(&self.pool)
            .await
            .context("catalog insert")?;
        Ok(result.rows_affected() > 0)
    }
}

// ---------------------------------------------------------------------------
// Catalog full-text search (change: score-hub-search) — gRPC-facing, so it
// returns the platform `Result`. Same thin-I/O glue, coverage-excluded and
// exercised by integration tests; the pure logic lives in the module + fakes.
// ---------------------------------------------------------------------------

use cymbra_platform::{AppError, Result as PlatformResult};

/// Maps a sqlx error to an internal `AppError` (no detail leaked to clients).
fn search_internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("catalog_search db: {e}"))
}

fn row_to_hit(r: &PgRow) -> CatalogHit {
    CatalogHit {
        id: r.get::<uuid::Uuid, _>("id").to_string(),
        title: r.get("title"),
        composer: r.get("composer"),
        level: r.get("level"),
        license: r.get("license"),
        source: r.get("source"),
        arranger: r.get("arranger"),
        min_note_value: r.get::<Option<i16>, _>("min_note_value").map(i32::from),
        tempo_bpm: r.get("tempo_bpm"),
        note_count: r.get("note_count"),
        lowest_midi: r.get::<Option<i16>, _>("lowest_midi").map(i32::from),
        highest_midi: r.get::<Option<i16>, _>("highest_midi").map(i32::from),
        time_sig: r.get("time_sig"),
        key_fifths: r.get("key_fifths"),
        // `needs_review` is a computed column present only on the moderation search;
        // absent elsewhere → default false. `moderation_status` is a real column in
        // `HIT_COLS`, so it is always present.
        needs_review: r.try_get("needs_review").unwrap_or(false),
        moderation_status: r.try_get::<String, _>("moderation_status").ok(),
    }
}

const HIT_COLS: &str = "id, title, composer, level, license, source, arranger, \
     min_note_value, tempo_bpm, note_count, lowest_midi, highest_midi, time_sig, key_fifths, \
     moderation_status";

/// The SQL ORDER BY expression for an allow-listed sort field, or `None` when the
/// key is unknown (already rejected by the module — defence in depth) and so
/// produces no ordering term; skipped keys are simply left out of the ORDER BY.
/// The expressions are constant and server-defined, so a field name never reaches
/// SQL unvalidated. `status_rank` ranks the queue (`pending` > `accepted` >
/// `rejected`); `needs_review` surfaces community-flagged scores (see
/// [`needs_review_sql`]).
fn sort_sql(field: &str) -> Option<String> {
    Some(match field {
        "measure_count" => "measure_count".to_string(),
        "staff_count" => "staff_count".to_string(),
        "note_count" => "note_count".to_string(),
        "min_note_value" => "min_note_value".to_string(),
        "tempo_bpm" => "tempo_bpm".to_string(),
        "title" => "title_norm".to_string(),
        "composer" => "composer_norm".to_string(),
        "status_rank" => {
            "CASE moderation_status WHEN 'pending' THEN 2 WHEN 'accepted' THEN 1 ELSE 0 END"
                .to_string()
        }
        "needs_review" => needs_review_sql(),
        // Any unknown field: no ORDER BY term.
        _ => return None,
    })
}

/// The ORDER BY expression flagging a score for moderator re-review — the SQL
/// mirror of [`is_flagged_for_review`](crate::score_rating::is_flagged_for_review)
/// now that the app-rating change (#2) supplies the data: the score has at least
/// `min_count` ratings AND their average effective value (explicit stars, else the
/// verdict's implied value: dislike 1.5 / like 3.5 / love 5) is at or below
/// `review_threshold`. Two correlated subqueries over `score_ratings` keyed by the
/// outer `catalog_scores.id`; ordering it DESC surfaces flagged scores first (a
/// boolean, so no bare non-integer constant that Postgres would reject). Thresholds
/// come from [`RatingConfig::default`](crate::score_rating::RatingConfig) so this
/// queue sort and the module's `needs_review` share one definition. Privileged (the
/// module gates the key to moderator/admin), so the per-row subqueries only run for
/// an authorised reviewer.
fn needs_review_sql() -> String {
    let cfg = crate::score_rating::RatingConfig::default();
    format!(
        "((SELECT COUNT(*) FROM music.score_ratings x \
            WHERE x.catalog_score_id = catalog_scores.id) >= {min_count} \
          AND COALESCE((SELECT AVG(CASE \
                WHEN x.stars IS NOT NULL THEN x.stars::float8 \
                WHEN x.verdict = 'dislike' THEN 1.5 \
                WHEN x.verdict = 'like' THEN 3.5 \
                WHEN x.verdict = 'love' THEN 5.0 END) \
             FROM music.score_ratings x \
             WHERE x.catalog_score_id = catalog_scores.id), 0) <= {threshold})",
        min_count = cfg.min_count,
        threshold = cfg.review_threshold,
    )
}

/// Build the ORDER BY body. Empty `sort` → the existing default (trigram similarity
/// on the `$1` query, then the `(title_norm, id)` stable tiebreak), byte-for-byte
/// unchanged so the app hub is unaffected. Non-empty → the validated keys (primary
/// first) followed by the same `(title_norm, id)` tiebreak for stable paging.
fn order_by_clause(sort: &[crate::catalog_search::SortKey]) -> String {
    if sort.is_empty() {
        return "CASE WHEN $1 = '' THEN 0 \
                     ELSE GREATEST(COALESCE(similarity(title_norm, $1), 0), \
                                   COALESCE(similarity(composer_norm, $1), 0)) END DESC, \
                title_norm ASC NULLS LAST, id ASC"
            .to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    for key in sort {
        if let Some(expr) = sort_sql(&key.field) {
            let dir = if key.descending { "DESC" } else { "ASC" };
            parts.push(format!("{expr} {dir} NULLS LAST"));
        }
    }
    parts.push("title_norm ASC NULLS LAST".to_string());
    parts.push("id ASC".to_string());
    parts.join(", ")
}

/// Postgres-backed [`CatalogSearchRepo`] over the `music_svc` pool.
pub struct PgCatalogSearchRepo {
    pool: PgPool,
}

impl PgCatalogSearchRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatalogSearchRepo for PgCatalogSearchRepo {
    async fn search(&self, p: &CatalogSearchParams) -> PlatformResult<(Vec<CatalogHit>, i64)> {
        // Substring match on the normalised columns (trigram-GIN accelerated),
        // ranked by trigram similarity with a deterministic `(title_norm, id)`
        // tiebreak so paging is stable. The query text/author are pre-normalised
        // by the module, so no re-parse/unaccent is needed at query time.
        // Facet filters ($4..$13): each is a no-op when its bind is NULL; when
        // set, a NULL facet column fails the predicate (unknown ⇒ excluded).
        // `COUNT(*) OVER()` reports the full match total (before LIMIT/OFFSET) on
        // every returned row — one round-trip, no separate count query.
        // Moderation gate ($13): `moderation_status = COALESCE($13, 'accepted')` —
        // a normal caller binds NULL and sees only `accepted`; a privileged caller
        // (authorised at the gRPC layer) binds an explicit status to select it
        // (change: add-score-moderation-gating). It composes with every filter above.
        // ORDER BY: an empty `sort` keeps the existing default (similarity, then the
        // `(title_norm, id)` tiebreak) so the app hub is unchanged; a non-empty
        // `sort` orders by the validated keys (constant, allow-listed expressions —
        // never raw input) followed by the same stable tiebreak (change:
        // add-moderation-back-office).
        let order_clause = order_by_clause(&p.sort);
        // The re-review predicate (design D4). Computed as a returned column only in a
        // privileged (back-office) context — a review-queue read, an explicit status
        // filter, or a moderation-oriented sort — so the app hub's search never pays
        // for the correlated subqueries (it selects a constant `false`).
        let flag_predicate = needs_review_sql();
        let privileged = p.review_queue
            || p.moderation_status.is_some()
            || p.sort
                .iter()
                .any(|k| crate::catalog_search::is_moderation_sort_field(&k.field));
        let review_flag_col = if privileged {
            flag_predicate.clone()
        } else {
            "false".to_string()
        };
        // Moderation gate. Review-queue mode ($16 = true): `pending` scores PLUS
        // `accepted` scores flagged for re-review — the moderation work list, ignoring
        // the single-status $13. Otherwise the single-status gate ($13, accepted-only
        // default) unchanged, so the app hub is unaffected.
        let rows = sqlx::query(&format!(
            "SELECT {HIT_COLS}, {review_flag_col} AS needs_review, \
                    COUNT(*) OVER() AS total_count FROM music.catalog_scores \
             WHERE ($1 = '' OR title_norm ILIKE '%'||$1||'%' OR composer_norm ILIKE '%'||$1||'%') \
               AND ($2::text IS NULL OR composer_norm ILIKE '%'||$2||'%') \
               AND ($3::text IS NULL OR level = $3) \
               AND ($4::bool  IS NULL OR is_piano = $4) \
               AND ($5::int2  IS NULL OR min_note_value <= $5) \
               AND ($6::bool  IS NULL OR has_chords = $6) \
               AND ($7::bool  IS NULL OR has_tuplets = $7) \
               AND ($8::bool  IS NULL OR has_dotted = $8) \
               AND ($9::int2  IS NULL OR highest_midi - lowest_midi <= $9) \
               AND ($10::int2 IS NULL OR staff_count = $10) \
               AND ($11::int4 IS NULL OR tempo_bpm >= $11) \
               AND ($12::int4 IS NULL OR tempo_bpm <= $12) \
               AND (($16::bool AND (moderation_status = 'pending' \
                       OR (moderation_status = 'accepted' AND {flag_predicate}))) \
                    OR (NOT $16::bool \
                        AND moderation_status = COALESCE($13::text, 'accepted'))) \
             ORDER BY {order_clause} \
             LIMIT $14 OFFSET $15"
        ))
        .bind(&p.text_norm)
        .bind(&p.author_norm)
        .bind(&p.level)
        .bind(p.facets.is_piano)
        .bind(p.facets.max_note_value)
        .bind(p.facets.has_chords)
        .bind(p.facets.has_tuplets)
        .bind(p.facets.has_dotted)
        .bind(p.facets.max_ambitus_semitones)
        .bind(p.facets.staff_count)
        .bind(p.facets.min_bpm)
        .bind(p.facets.max_bpm)
        .bind(&p.moderation_status)
        .bind(p.limit)
        .bind(p.offset)
        .bind(p.review_queue)
        .fetch_all(&self.pool)
        .await
        .map_err(search_internal)?;
        // The window count is identical on every row; an empty page (e.g. offset
        // past the end) yields no rows, so the total is 0 there.
        let total = rows
            .first()
            .map(|r| r.get::<i64, _>("total_count"))
            .unwrap_or(0);
        let hits = rows.iter().map(row_to_hit).collect();
        Ok((hits, total))
    }

    async fn hits_by_ids(&self, ids: &[String]) -> PlatformResult<Vec<CatalogHit>> {
        // Skip unparseable ids (never a 500); an empty set short-circuits.
        let uuids: Vec<uuid::Uuid> = ids
            .iter()
            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
            .collect();
        if uuids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query(&format!(
            "SELECT {HIT_COLS} FROM music.catalog_scores WHERE id = ANY($1)"
        ))
        .bind(&uuids)
        .fetch_all(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(rows.iter().map(row_to_hit).collect())
    }

    async fn hit_by_id(
        &self,
        id: &str,
        include_unvalidated: bool,
    ) -> PlatformResult<Option<CatalogHit>> {
        let Ok(uuid) = uuid::Uuid::parse_str(id) else {
            return Ok(None); // malformed id → not found, never a 500
        };
        // Same moderation gate ($2) as `object_key`: normal caller sees `accepted`
        // only; an authorised reviewer (`true`) sees any status.
        let row = sqlx::query(&format!(
            "SELECT {HIT_COLS} FROM music.catalog_scores \
             WHERE id = $1 AND ($2 OR moderation_status = 'accepted')"
        ))
        .bind(uuid)
        .bind(include_unvalidated)
        .fetch_optional(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(row.as_ref().map(row_to_hit))
    }

    async fn object_key(
        &self,
        id: &str,
        include_unvalidated: bool,
    ) -> PlatformResult<Option<String>> {
        let Ok(uuid) = uuid::Uuid::parse_str(id) else {
            return Ok(None); // malformed id → not found, never a 500
        };
        // Moderation gate ($2): a normal caller (`include_unvalidated = false`) only
        // resolves an `accepted` score, so a `pending`/`rejected` id is reported as
        // absent — its bytes can't be fetched and it can't be saved. An authorised
        // reviewer (`true`) resolves any status (change: add-score-moderation-gating).
        let row = sqlx::query(
            "SELECT object_key FROM music.catalog_scores \
             WHERE id = $1 AND ($2 OR moderation_status = 'accepted')",
        )
        .bind(uuid)
        .bind(include_unvalidated)
        .fetch_optional(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(row.map(|r| r.get::<String, _>("object_key")))
    }

    async fn set_moderation_status(
        &self,
        score_id: &str,
        status: &str,
        reviewer_id: &str,
    ) -> PlatformResult<bool> {
        let Ok(id) = uuid::Uuid::parse_str(score_id) else {
            return Ok(false); // malformed id → not found (no row updated)
        };
        // A reviewer id that isn't a UUID is a caller/programming error, not a
        // client 500: treat it as no-op not-found rather than failing the query.
        let Ok(reviewer) = uuid::Uuid::parse_str(reviewer_id) else {
            return Ok(false);
        };
        // Single conditional UPDATE: set the status and stamp reviewer + time in one
        // write (change: add-moderation-back-office). rows_affected tells us whether
        // the score existed.
        let result = sqlx::query(
            "UPDATE music.catalog_scores \
             SET moderation_status = $2, reviewed_by = $3, reviewed_at = now() \
             WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(reviewer)
        .execute(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(result.rows_affected() > 0)
    }

    async fn apply_metadata_edit(
        &self,
        score_id: &str,
        editor: &str,
        plan: &crate::catalog_edit::EditPlan,
    ) -> PlatformResult<bool> {
        let Ok(id) = uuid::Uuid::parse_str(score_id) else {
            return Ok(false); // malformed id → not found
        };
        let Ok(editor_id) = uuid::Uuid::parse_str(editor) else {
            return Ok(false); // non-UUID editor is a programming error, not a 500
        };

        // One transaction: the row update + the recomputed search keys + provenance,
        // then one audit row per changed field (change: add-catalog-metadata-editing).
        let mut tx = self.pool.begin().await.map_err(search_internal)?;

        // `level_source` becomes 'manual' only when the level actually changed. The
        // recomputed title_norm/composer_norm/work_key keep search consistent with the
        // edit; edited_by/edited_at mark the row as manually edited (refresh-skip).
        let updated = sqlx::query(
            "UPDATE music.catalog_scores SET \
                title = $2, composer = $3, arranger = $4, level = $5, \
                title_norm = $6, composer_norm = $7, work_key = $8, \
                level_source = CASE WHEN $9 THEN 'manual' ELSE level_source END, \
                edited_by = $10, edited_at = now() \
             WHERE id = $1",
        )
        .bind(id)
        .bind(&plan.title)
        .bind(&plan.composer)
        .bind(&plan.arranger)
        .bind(&plan.level)
        .bind(&plan.title_norm)
        .bind(&plan.composer_norm)
        .bind(&plan.work_key)
        .bind(plan.level_changed())
        .bind(editor_id)
        .execute(&mut *tx)
        .await
        .map_err(search_internal)?;

        if updated.rows_affected() == 0 {
            tx.rollback().await.map_err(search_internal)?;
            return Ok(false);
        }

        for c in &plan.changes {
            sqlx::query(
                "INSERT INTO music.catalog_edits \
                    (catalog_score_id, editor, field, old_value, new_value) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind(editor_id)
            .bind(c.field)
            .bind(&c.old)
            .bind(&c.new)
            .execute(&mut *tx)
            .await
            .map_err(search_internal)?;
        }

        tx.commit().await.map_err(search_internal)?;
        Ok(true)
    }

    async fn rating_deck(
        &self,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> PlatformResult<Vec<CatalogHit>> {
        let Ok(user) = uuid::Uuid::parse_str(user_id) else {
            return Ok(Vec::new()); // malformed identity → nothing to rate
        };
        // The caller's un-rated `pending` + `accepted` scores, least-rated first —
        // `rejected` is never sourced (change: rate-pending-scores). A LEFT JOIN to
        // the caller's own ratings + `r.user_id IS NULL` excludes what they already
        // rated; the correlated COUNT orders by how many ratings each score has
        // (fewest first — those most need signal), with an `id` tiebreak for stable
        // paging (change: improve-rating-deck-sourcing).
        let rows = sqlx::query(&format!(
            "SELECT {HIT_COLS} FROM music.catalog_scores cs \
             LEFT JOIN music.score_ratings r \
               ON r.catalog_score_id = cs.id AND r.user_id = $1 \
             WHERE cs.moderation_status IN ('pending', 'accepted') AND cs.is_piano \
               AND r.user_id IS NULL \
             ORDER BY (SELECT COUNT(*) FROM music.score_ratings x \
                       WHERE x.catalog_score_id = cs.id) ASC, cs.id ASC \
             LIMIT $2 OFFSET $3"
        ))
        .bind(user)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(rows.iter().map(row_to_hit).collect())
    }
}

// ---------------------------------------------------------------------------
// Score ratings (change: add-app-score-rating) — gRPC-facing, so it returns the
// platform `Result`. Thin I/O glue (coverage-excluded); the rating math lives in
// `score_rating.rs` + its fake, and this adapter mirrors the same arithmetic in
// SQL (the effective value: explicit stars, else the verdict's implied value).
// ---------------------------------------------------------------------------

use crate::score_rating::{RatingAggregate, ScoreRatingRepo, Verdict};

/// Postgres-backed [`ScoreRatingRepo`] over the `music_svc` pool.
pub struct PgScoreRatingRepo {
    pool: PgPool,
}

impl PgScoreRatingRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScoreRatingRepo for PgScoreRatingRepo {
    async fn upsert(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        verdict: Verdict,
        stars: Option<i16>,
    ) -> PlatformResult<()> {
        // Both ids are UUIDs (the user from auth, the score already resolved
        // through the accepted-only path in the module). A malformed id here is a
        // programming error, not client input — surface it as internal rather than
        // panicking.
        let user = uuid::Uuid::parse_str(user_id)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("score_rating bad user id: {e}")))?;
        let score = uuid::Uuid::parse_str(catalog_score_id)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("score_rating bad score id: {e}")))?;
        // Upsert: one row per (user, score); a re-rating overwrites verdict/stars
        // and bumps updated_at.
        sqlx::query(
            "INSERT INTO music.score_ratings (user_id, catalog_score_id, verdict, stars, updated_at) \
             VALUES ($1, $2, $3, $4, now()) \
             ON CONFLICT (user_id, catalog_score_id) \
             DO UPDATE SET verdict = EXCLUDED.verdict, stars = EXCLUDED.stars, updated_at = now()",
        )
        .bind(user)
        .bind(score)
        .bind(verdict.as_str())
        .bind(stars)
        .execute(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(())
    }

    async fn aggregate(&self, catalog_score_id: &str) -> PlatformResult<RatingAggregate> {
        let Ok(score) = uuid::Uuid::parse_str(catalog_score_id) else {
            return Ok(RatingAggregate::default()); // malformed id → no ratings
        };
        // The effective value per rating mirrors `score_rating::effective_value`:
        // explicit stars, else the verdict's implied value (dislike 1.5 / like 3.5
        // / love 5). AVG over the empty set is NULL → COALESCE to 0. The verdict
        // breakdown uses FILTER counts in the same single scan.
        let row = sqlx::query(
            "SELECT \
                COUNT(*) AS cnt, \
                COALESCE(AVG(CASE \
                    WHEN stars IS NOT NULL THEN stars::float8 \
                    WHEN verdict = 'dislike' THEN 1.5 \
                    WHEN verdict = 'like' THEN 3.5 \
                    WHEN verdict = 'love' THEN 5.0 END), 0) AS avg_effective, \
                COUNT(*) FILTER (WHERE verdict = 'dislike') AS dislike, \
                COUNT(*) FILTER (WHERE verdict = 'like')    AS like_cnt, \
                COUNT(*) FILTER (WHERE verdict = 'love')    AS love \
             FROM music.score_ratings WHERE catalog_score_id = $1",
        )
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(RatingAggregate {
            avg_effective: row.get::<f64, _>("avg_effective"),
            count: row.get::<i64, _>("cnt"),
            dislike: row.get::<i64, _>("dislike"),
            like: row.get::<i64, _>("like_cnt"),
            love: row.get::<i64, _>("love"),
        })
    }

    async fn count_recent_by_user(
        &self,
        user_id: &str,
        window: std::time::Duration,
    ) -> PlatformResult<u64> {
        let Ok(user) = uuid::Uuid::parse_str(user_id) else {
            return Ok(0); // malformed id → no ratings
        };
        let row = sqlx::query(
            "SELECT COUNT(*) AS cnt FROM music.score_ratings \
             WHERE user_id = $1 AND updated_at >= now() - make_interval(secs => $2)",
        )
        .bind(user)
        .bind(window.as_secs_f64())
        .fetch_one(&self.pool)
        .await
        .map_err(search_internal)?;
        Ok(row.get::<i64, _>("cnt").max(0) as u64)
    }
}

// ---------------------------------------------------------------------------
// Title backfill (recompute catalog titles from stored bytes) — thin SQL glue for
// [`crate::backfill`]. `anyhow`-based like the ingestion path (run by a one-off
// maintenance binary, not over gRPC). Coverage-excluded; the paging/keyset logic
// and the plan/apply orchestration are host-tested with fakes in `backfill.rs`.
// ---------------------------------------------------------------------------

use crate::backfill::{BackfillRow, TitleBackfillRepo, TitleUpdate};

/// Postgres-backed [`TitleBackfillRepo`] over an ingestion/admin pool.
pub struct PgTitleBackfillRepo {
    pool: PgPool,
}

impl PgTitleBackfillRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TitleBackfillRepo for PgTitleBackfillRepo {
    async fn page(
        &self,
        after: &str,
        source: Option<&str>,
        limit: i64,
    ) -> Result<Vec<BackfillRow>> {
        // Keyset paging on the UUID PK: `id > $1` (NULL $1 → from the start),
        // ordered by `id` for a stable, resumable scan. An optional `source` scopes
        // the sweep (e.g. only `openscore`). `after` is a UUID text; an unparseable
        // value (including "") means "from the start".
        //
        // Anti-clobber (change: add-catalog-metadata-editing): skip rows that carry
        // the manual-edit marker (`edited_by IS NOT NULL`) so this automated
        // metadata-refresh path never overwrites a moderator's correction.
        let after_uuid = uuid::Uuid::parse_str(after).ok();
        let rows = sqlx::query(
            "SELECT id, object_key, title FROM music.catalog_scores \
             WHERE ($1::uuid IS NULL OR id > $1) \
               AND ($2::text IS NULL OR source = $2) \
               AND edited_by IS NULL \
             ORDER BY id ASC LIMIT $3",
        )
        .bind(after_uuid)
        .bind(source)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("backfill page")?;
        Ok(rows
            .iter()
            .map(|r| BackfillRow {
                id: r.get::<uuid::Uuid, _>("id").to_string(),
                object_key: r.get("object_key"),
                title: r.get("title"),
            })
            .collect())
    }

    async fn update_title(&self, id: &str, update: &TitleUpdate) -> Result<()> {
        let uuid = uuid::Uuid::parse_str(id).context("backfill update: bad id")?;
        // Rewrite the display title AND its search key + work key together, so
        // `title`/`title_norm`/`work_key` stay mutually consistent (search matches
        // `title_norm`, never the display `title`).
        sqlx::query(
            "UPDATE music.catalog_scores \
             SET title = $2, title_norm = $3, work_key = $4 \
             WHERE id = $1",
        )
        .bind(uuid)
        .bind(&update.title)
        .bind(&update.title_norm)
        .bind(&update.work_key)
        .execute(&self.pool)
        .await
        .context("backfill update_title")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ingestion can only ever persist a `pending` score: the INSERT hardcodes the
    /// `moderation_status` literal (never a bind), so no caller — regardless of
    /// licensing `confidence` — can auto-validate a crawled row (change:
    /// add-score-moderation-gating). This guards the D5 invariant without a DB; the
    /// end-to-end behaviour is covered by the integration/migration checks.
    #[test]
    fn catalog_insert_hardcodes_pending_moderation_status() {
        let sql = catalog_insert_sql();
        // The column is inserted, with the literal value — and there is no `$…`
        // bind that could carry a different status in.
        assert!(sql.contains("moderation_status"));
        assert!(sql.contains("'pending'"));
        assert!(
            !sql.contains("'accepted'") && !sql.contains("'rejected'"),
            "insert must never write accepted/rejected"
        );
    }

    use crate::catalog_search::SortKey;

    fn key(field: &str) -> SortKey {
        SortKey {
            field: field.into(),
            descending: true,
        }
    }

    /// The queue's default sort WIRES `needs_review` now that #2 supplies rating
    /// data: it emits a correlated re-review predicate over `score_ratings` as the
    /// primary key (a boolean — no bare non-integer constant that Postgres would
    /// reject in ORDER BY), carrying the default thresholds, with the other keys +
    /// the stable tiebreak following.
    #[test]
    fn order_by_wires_needs_review_flag_first() {
        let clause = order_by_clause(&[
            key("needs_review"),
            key("status_rank"),
            key("measure_count"),
            key("staff_count"),
        ]);
        // needs_review is now a real correlated predicate over the ratings table,
        // not a skipped no-op, and it leads the ORDER BY.
        assert!(
            clause.contains("music.score_ratings"),
            "needs_review must query the ratings table: {clause}"
        );
        assert!(
            clause.trim_start().starts_with("(("),
            "the re-review predicate must be the primary sort key: {clause}"
        );
        // Carries the default N/T thresholds (single source: RatingConfig::default).
        let cfg = crate::score_rating::RatingConfig::default();
        assert!(clause.contains(&format!(">= {}", cfg.min_count)));
        assert!(clause.contains(&format!("<= {}", cfg.review_threshold)));
        // No bare non-integer constant that Postgres would reject in ORDER BY.
        assert!(
            !clause.contains("false"),
            "must not emit `false` in ORDER BY: {clause}"
        );
        // The backed keys + the deterministic tiebreak are ordered as expected.
        assert!(clause.contains("moderation_status")); // status_rank → CASE
        assert!(clause.contains("measure_count DESC"));
        assert!(clause.contains("staff_count DESC"));
        assert!(clause.ends_with("title_norm ASC NULLS LAST, id ASC"));
    }

    /// An empty sort keeps the pre-existing default ordering (hub unaffected).
    #[test]
    fn order_by_empty_sort_uses_similarity_default() {
        let clause = order_by_clause(&[]);
        assert!(clause.contains("similarity(title_norm, $1)"));
        assert!(clause.ends_with("title_norm ASC NULLS LAST, id ASC"));
    }
}
