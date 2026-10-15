-- Copyright 2026 NEETROF
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not use
-- this file except in compliance with the License. You may obtain a copy of the
-- License at http://www.apache.org/licenses/LICENSE-2.0
--
-- Anonymous usage counts (change: add-lingua-usage-counts, D9): counters of an
-- installation's six steps, keyed by its installation day, and week counters of the leaving
-- page's six answers. Counters only — there is no column for an address, a header, a time of
-- day, an account or a report, and no row is ever written per report. Counters more than 25
-- months old are deleted by the worker's daily `lingua_usage_prune` job. Numbered 0010: 0009
-- is planned by add-lingua-card-sentence-translation (sqlx applies every migration it has not
-- applied, whatever the order they land in). Idempotent and schema-qualified like 0001-0008.

CREATE TABLE IF NOT EXISTS lingua.usage_counts_daily (
  install_day INTEGER NOT NULL,  -- day number (days since 1970-01-01) of day 0, the installation's local calendar day
  step        TEXT    NOT NULL,  -- installed | set_up | first_analysis | returned_day_1 | returned_days_7_13 | returned_days_30_59
  browser     TEXT    NOT NULL,  -- chromium | firefox | safari
  version     TEXT    NOT NULL,  -- major.minor | other
  count       BIGINT  NOT NULL DEFAULT 0,
  PRIMARY KEY (install_day, step, browser, version)
);

CREATE TABLE IF NOT EXISTS lingua.leave_reasons_weekly (
  week    INTEGER NOT NULL,       -- day number of the Monday of the UTC week the answer arrived in
  reason  TEXT    NOT NULL,       -- not_using | highlighting | missing_language | slow | unclear | other
  browser TEXT    NOT NULL,       -- chromium | firefox | safari | unknown
  version TEXT    NOT NULL,       -- major.minor | other | unknown
  count   BIGINT  NOT NULL DEFAULT 0,
  PRIMARY KEY (week, reason, browser, version)
);
