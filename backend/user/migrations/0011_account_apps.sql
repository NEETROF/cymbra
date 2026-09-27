-- The apps an account has signed in to (change: add-directory-app-usage).
--
-- `auth.sessions` carries the audience but is deleted on sign-out, revocation and 30
-- days after the last refresh, so it answers "signed in now", not "uses this app". This
-- is the durable fact the back-office directory shows and filters on. It lives here,
-- not in `auth`, because the directory query runs as `user_svc` (search_path =
-- `user_account`); the auth module writes it through the UserPort on every token mint.
--
-- `last_seen_at` is only rewritten when it is over an hour old, so a refresh loop costs
-- no write. Deleted with the account (and by the orphan reaper) via the cascade — the
-- FK is schema-qualified so the erasure guard (worker/tests/erasure_coverage.rs) sees it.
--
-- The CHECK restates the app scopes (`cymbra_platform::APP_SCOPES`); a unit test fails
-- if the two drift, like the role vocabulary of 0010.

CREATE TABLE account_apps (
    user_id       UUID        NOT NULL REFERENCES user_account.users (id) ON DELETE CASCADE,
    app           TEXT        NOT NULL CHECK (app IN ('music', 'live', 'lingua')),
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, app)
);

-- The directory's app filter: "accounts with a row for app X".
CREATE INDEX account_apps_app_idx ON account_apps (app, user_id);
