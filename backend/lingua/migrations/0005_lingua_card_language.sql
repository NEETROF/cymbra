-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- A deck card carries its studied language (change: add-lingua-card-language). Every
-- row written before this migration came from a client that only ever held English
-- cards, so the default `en` is the value those clients would have sent and no row is
-- rewritten. The primary key gains the language: the same client id in two languages is
-- two cards. Idempotent and schema-qualified like 0001-0004.

ALTER TABLE lingua.cards ADD COLUMN IF NOT EXISTS language TEXT NOT NULL DEFAULT 'en';

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    JOIN pg_namespace n ON n.oid = t.relnamespace
    WHERE n.nspname = 'lingua' AND t.relname = 'cards' AND c.contype = 'p'
      AND c.conkey = (
        SELECT array_agg(a.attnum ORDER BY x.ord)
        FROM unnest(ARRAY['user_id', 'language', 'client_id']) WITH ORDINALITY AS x(name, ord)
        JOIN pg_attribute a ON a.attrelid = t.oid AND a.attname = x.name
      )
  ) THEN
    ALTER TABLE lingua.cards DROP CONSTRAINT IF EXISTS cards_pkey;
    ALTER TABLE lingua.cards ADD PRIMARY KEY (user_id, language, client_id);
  END IF;
END $$;
