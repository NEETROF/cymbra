-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- A card carries the language of its gloss and a daily statistic the native language
-- of its device (change: add-lingua-native-language-server). Every row written before
-- this migration came from a client whose packs are glossed in French, so the default
-- `fr` labels those rows exactly; Postgres adds a defaulted column without rewriting a
-- row. No key changes: a card is still (user, language, client id) and a statistic
-- (user, day, language, device) — the new columns are values of the row. Idempotent
-- and schema-qualified like 0001-0005.

ALTER TABLE lingua.cards
  ADD COLUMN IF NOT EXISTS gloss_language TEXT NOT NULL DEFAULT 'fr';

ALTER TABLE lingua.daily_stats
  ADD COLUMN IF NOT EXISTS native_language TEXT NOT NULL DEFAULT 'fr';
