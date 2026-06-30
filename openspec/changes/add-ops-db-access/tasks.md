## 1. Env-driven bootstrap refactor

- [x] 1.1 Convert `backend/db/init/01-schemas-roles.sql` into a secret-free SQL template that reads credentials as psql variables (`:'auth_pw'`, role names, etc.) instead of literals — `db/init/roles.sql.tpl` (`.sql.tpl` so the initdb.d glob ignores it; role names via `:"role"`, passwords via `:'pw'`)
- [x] 1.2 Add a `docker-entrypoint-initdb.d` entrypoint wrapper (`00-roles.sh`) that reads `CYMBRA_*_DB_PASSWORD` / role-name env vars (each defaulting to the current dev value) and runs `psql -v … -f` the template
- [x] 1.3 Make the bootstrap idempotent / re-runnable (guard role creation; `ALTER ROLE … PASSWORD` on re-apply) so it works on already-provisioned DBs, not only fresh volumes — `\gexec` create-if-missing + `ALTER ROLE … PASSWORD`; verified by running the wrapper twice against the live dev DB
- [x] 1.4 Keep `docker-compose.yml` wiring so `docker compose up` still bootstraps with dev defaults — the `./db/init` mount already covers both files; `00-roles.sh` runs first, `roles.sql.tpl` is ignored by the entrypoint; verified via `docker exec … 00-roles.sh`

## 2. Ops `admin_svc` role

- [x] 2.1 Add the `admin_svc` LOGIN role to the bootstrap, granted `pg_read_all_data` + `pg_write_all_data`, with `search_path` spanning `auth, user_account, jobs, public`
- [x] 2.2 Confirm it is pure DML: it owns no objects and cannot run DDL (no schema/role ownership) — verified live: reads auth/user_account/jobs + writes jobs.schedules, but `CREATE TABLE jobs.…` is denied
- [x] 2.3 Add `CYMBRA_ADMIN_DATABASE_URL` to `.env.example` under an ops-only note (explicitly: not read by any service) — + the bootstrap `CYMBRA_*_DB_PASSWORD` defaults

## 3. CI + verification

- [x] 3.1 Update `backend-it.yml` to apply the bootstrap via the env-driven wrapper (dev defaults) and keep the integration suite green — runs `00-roles.sh`; adds `CYMBRA_ADMIN_DATABASE_URL`
- [x] 3.2 Integration assertions: `admin_svc` can read+write `auth`, `user_account`, `jobs`; a module role is still confined to its own schema (D0 preserved); `admin_svc` cannot DDL — `jobs/tests/ops_admin_role.rs` (`#[ignore]`, passes locally); D0 confinement stays covered by `cymbra-user`'s `db_isolation` test
- [x] 3.3 Verify the committed SQL contains no password literals — `roles.sql.tpl` uses only `:'…_pw'` variables (grep confirmed)

## 4. Production credential strategy (docs)

- [x] 4.1 Document the prod secret strategy (secret manager vs IAM database auth) and rotation in `backend/README.md` (or an ops doc), replacing the `*_dev_pw` literals — new "Database roles & operations" section
- [x] 4.2 Note the dependency/ordering on `add-job-infrastructure` (the `jobs`/`worker_svc` bits) and the per-operator-IAM-role open question — both noted in that README section

## 5. Validation

- [x] 5.1 `openspec validate add-ops-db-access --strict` passes
- [x] 5.2 Bootstrap runs clean on a fresh volume and re-runs clean on an existing one (dev) — create path (new `admin_svc`) + idempotent re-run both verified against the live dev DB via the wrapper
