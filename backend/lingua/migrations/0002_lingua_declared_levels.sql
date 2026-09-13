-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- Declared CEFR level sync (change: add-lingua-cefr-levels). A per-(user, language)
-- preference that rides KnownWordsService: last-write-wins like word_statuses, sharing
-- the same lingua.change_seq so a level change advances the same per-user cursor/ETag
-- and propagates with statuses. Idempotent + schema-qualified like 0001.
--
-- PRIVACY ALLOW-LIST: the declared level is the user's own learning preference. No
-- browsing URL, page text, per-page counter, or reading history — the 0001 allow-list holds.
--
-- GDPR: erased by user_id in backend/worker (purge_user_with) alongside the other lingua
-- tables — the purge block lists this table name too.

-- Declared level: last-write-wins per (user, language). `level` is '' for "débutant"
-- (an explicit from-zero choice) or a CEFR label 'A1'..'C2'.
CREATE TABLE IF NOT EXISTS lingua.declared_levels (
  user_id UUID NOT NULL,
  language TEXT NOT NULL,
  level TEXT NOT NULL,                  -- '' = débutant | 'A1'..'C2'
  updated_at BIGINT NOT NULL,          -- winning decision timestamp (epoch millis, future-clamped)
  device_id TEXT NOT NULL DEFAULT '',  -- LWW tie-break when timestamps are equal
  seq BIGINT NOT NULL,                 -- shares lingua.change_seq with word_statuses
  PRIMARY KEY (user_id, language)
);
CREATE INDEX IF NOT EXISTS declared_levels_cursor ON lingua.declared_levels (user_id, seq);
