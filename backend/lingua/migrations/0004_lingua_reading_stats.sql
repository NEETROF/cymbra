-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- Viewport-gated reading stats (change: refine-lingua-reading-stats). Idempotent and
-- schema-qualified like 0001-0003. `unknown_seen` = new words seen among the words read
-- (`exposures`). Only stats that report it are stored from now on, so every row written
-- after this migration carries a reported value; rows already there keep their old
-- whole-document figures and read 0 (test history — no data is rewritten).

ALTER TABLE lingua.daily_stats ADD COLUMN IF NOT EXISTS unknown_seen INTEGER NOT NULL DEFAULT 0;
