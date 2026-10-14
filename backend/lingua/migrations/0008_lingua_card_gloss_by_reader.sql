-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- Whether the reader chose a card's gloss — a machine translation they kept in review, or a
-- translation they wrote (changes: add-lingua-review-translation-fallback D6, shared with
-- add-lingua-card-edit). A value of the row, stored and returned as received: every row
-- written before this migration came from a client that had no such choice, so the default
-- `false` is what those clients would have sent and no row is rewritten. Idempotent and
-- schema-qualified like 0001-0007.

ALTER TABLE lingua.cards
  ADD COLUMN IF NOT EXISTS gloss_by_reader BOOLEAN NOT NULL DEFAULT false;
