-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- Cymbra Lingua schema (change: add-lingua-backend). Idempotent, fully schema-qualified
-- so a double-apply is safe regardless of the connecting role's search_path. Every table
-- is keyed by `user_id` (a plain UUID — NO cross-schema FK to user_account; account state
-- comes through the UserPort). GDPR erasure deletes these rows by user_id in
-- backend/worker (purge_user_with); the table names here MUST match that block.
--
-- PRIVACY ALLOW-LIST: only lemma statuses, user-created cards, and day-grained learning
-- aggregates. No browsing URL, page text, per-page counter, or reading history.

-- A per-user monotonic change sequence drives the cursor pull for statuses and cards.
CREATE SEQUENCE IF NOT EXISTS lingua.change_seq;

-- Word statuses: last-write-wins per (user, language, lemma).
CREATE TABLE IF NOT EXISTS lingua.word_statuses (
  user_id UUID NOT NULL,
  language TEXT NOT NULL,
  lemma TEXT NOT NULL,
  status TEXT NOT NULL,
  provenance TEXT NOT NULL DEFAULT '',
  updated_at BIGINT NOT NULL,          -- winning op timestamp (epoch millis, future-clamped)
  device_id TEXT NOT NULL DEFAULT '',  -- LWW tie-break when timestamps are equal
  seq BIGINT NOT NULL,                 -- monotonic change sequence (nextval on every write)
  PRIMARY KEY (user_id, language, lemma)
);
CREATE INDEX IF NOT EXISTS word_statuses_cursor ON lingua.word_statuses (user_id, seq);

-- Deck cards: whole cards, last-write-wins per (user, client_id), tombstones included.
-- Media contents are never stored (allow-list); there is no media column.
CREATE TABLE IF NOT EXISTS lingua.cards (
  user_id UUID NOT NULL,
  client_id TEXT NOT NULL,
  lemma TEXT NOT NULL,
  surface_form TEXT NOT NULL DEFAULT '',
  source_sentence TEXT NOT NULL DEFAULT '',
  source TEXT NOT NULL DEFAULT '',      -- the origin the user attached to the card (their own data)
  gloss TEXT,
  fsrs_state TEXT NOT NULL DEFAULT '',
  deleted BOOLEAN NOT NULL DEFAULT FALSE,
  updated_at BIGINT NOT NULL,
  device_id TEXT NOT NULL DEFAULT '',
  seq BIGINT NOT NULL,
  PRIMARY KEY (user_id, client_id)
);
CREATE INDEX IF NOT EXISTS cards_cursor ON lingua.cards (user_id, seq);

-- Daily learning aggregates: idempotent upsert per (user, UTC day, language, device);
-- reads SUM across devices. Day × language is the finest grain — no hour, no source.
CREATE TABLE IF NOT EXISTS lingua.daily_stats (
  user_id UUID NOT NULL,
  day INTEGER NOT NULL,                 -- UTC day key (days since the Unix epoch)
  language TEXT NOT NULL,
  device_id TEXT NOT NULL,
  exposures INTEGER NOT NULL DEFAULT 0,
  words_learned INTEGER NOT NULL DEFAULT 0,
  reviews_done INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (user_id, day, language, device_id)
);
CREATE INDEX IF NOT EXISTS daily_stats_range ON lingua.daily_stats (user_id, day);
