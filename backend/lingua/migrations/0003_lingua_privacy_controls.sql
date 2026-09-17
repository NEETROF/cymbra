-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- Lingua privacy controls (change: add-lingua-privacy-controls). Idempotent and
-- schema-qualified like 0001/0002.
--
-- 1. A card's page address stays on the device: the column goes, and with it every
--    address stored so far (intended — no backup copy is kept here). The wire field
--    `CardOp.source` stays, deprecated and ignored.
-- 2. The Lingua-only erasure mark: `LinguaDataService.EraseMyData` deletes the user's
--    lingua rows and records the server time here; pushes dated at or before it are
--    dropped, and every client reads it before pushing (design D2/D3).
--
-- GDPR: erased by user_id in backend/worker (purge_user_with) alongside the other lingua
-- tables — the purge block lists this table name too.

ALTER TABLE lingua.cards DROP COLUMN IF EXISTS source;

CREATE TABLE IF NOT EXISTS lingua.data_erasures (
  user_id UUID PRIMARY KEY,
  erased_at BIGINT NOT NULL            -- server time of the latest erasure (epoch millis)
);
