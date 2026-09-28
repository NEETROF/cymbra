## Context

The directory row (`AccountRow`) already carries the account's apps with their last use
(`add-directory-app-usage`). `users.created_at` exists since migration `0003` (2026-06-29)
but is not exposed. The account page loads its account through the same `ListAccounts`
call (`ids = [userId]`), so one row shape feeds both screens.

## Goals / Non-Goals

**Goals:** show sign-up date and last sign-in in the Users table and on the account page.

**Non-Goals:** a true interactive sign-in timestamp (the auth module would have to write
a new field, with no history); sorting or filtering by these dates; backfilling the
sign-up date of pre-`0003` accounts.

## Decisions

- **D1 — Sign-up date is `users.created_at`, no fallback.** The account id is a UUID v7
  and embeds the creation instant, but only accounts older than 2026-06-29 would differ,
  and they are few. One source, no id parsing.
- **D2 — Last sign-in is derived in the back office from `AccountRow.apps`.** The max of
  the apps' `last_seen_at` is already on the row. A dedicated `last_sign_in_at` field
  would duplicate it and could drift from the icons' tooltips. Alternative rejected: a
  `max()` subquery on the server — same value, more wire and more code.
- **D3 — One date helper.** Both views format a unix-seconds `bigint` the same way
  (`toLocaleDateString(currentLocale())`, as the app tooltip already does); the helper
  lives in `src/lib/` and the tooltip uses it too.

## Risks / Trade-offs

- [Pre-`0003` accounts show 2026-06-29] → accepted (D1).
- [Last sign-in counts refreshes and is coarsened to the hour] → the value is a date,
  not a time, so the hour does not show; "activity" is what an operator wants anyway.
- [Accounts idle since 2026-09-27 show a dash] → fills as they return.

## Migration Plan

Additive proto field; old back office builds ignore it. No schema change. Rollback = revert.
