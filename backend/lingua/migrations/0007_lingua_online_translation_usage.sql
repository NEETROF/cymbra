-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- « Traduction en ligne » usage (change: add-lingua-online-translation, D9): daily totals per
-- provider, pair and outcome, for the back office's figures and cost. No account and no text
-- by construction — there is no column for either. The cost is estimated at request time from
-- the configured prices, so a later price change never rewrites a past day. Idempotent and
-- schema-qualified like 0001-0006.

CREATE TABLE IF NOT EXISTS lingua.online_translation_daily (
  day             INTEGER NOT NULL,           -- days since the epoch (UTC)
  provider        TEXT    NOT NULL,           -- gemini | anthropic | none (refused first)
  pair            TEXT    NOT NULL,           -- <source>-<target>, e.g. es-fr
  outcome         TEXT    NOT NULL,           -- answered | failed | limited
  requests        BIGINT  NOT NULL DEFAULT 0,
  input_tokens    BIGINT  NOT NULL DEFAULT 0,
  output_tokens   BIGINT  NOT NULL DEFAULT 0,
  cost_micro_usd  BIGINT  NOT NULL DEFAULT 0,
  PRIMARY KEY (day, provider, pair, outcome)
);
