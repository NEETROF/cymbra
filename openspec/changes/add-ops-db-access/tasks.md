## 1. Env-driven bootstrap refactor

- [ ] 1.1 Convert `backend/db/init/01-schemas-roles.sql` into a secret-free SQL template that reads credentials as psql variables (`:'auth_pw'`, role names, etc.) instead of literals
- [ ] 1.2 Add a `docker-entrypoint-initdb.d` entrypoint wrapper (`00-roles.sh`) that reads `CYMBRA_*_DB_PASSWORD` / role-name env vars (each defaulting to the current dev value) and runs `psql -v … -f` the template
- [ ] 1.3 Make the bootstrap idempotent / re-runnable (guard role creation; `ALTER ROLE … PASSWORD` on re-apply) so it works on already-provisioned DBs, not only fresh volumes
- [ ] 1.4 Keep `docker-compose.yml` wiring so `docker compose up` still bootstraps with dev defaults

## 2. Ops `admin_svc` role

- [ ] 2.1 Add the `admin_svc` LOGIN role to the bootstrap, granted `pg_read_all_data` + `pg_write_all_data`, with `search_path` spanning `auth, user_account, jobs, public`
- [ ] 2.2 Confirm it is pure DML: it owns no objects and cannot run DDL (no schema/role ownership)
- [ ] 2.3 Add `CYMBRA_ADMIN_DATABASE_URL` to `.env.example` under an ops-only note (explicitly: not read by any service)

## 3. CI + verification

- [ ] 3.1 Update `backend-it.yml` to apply the bootstrap via the env-driven wrapper (dev defaults) and keep the integration suite green
- [ ] 3.2 Integration assertions: `admin_svc` can read+write `auth`, `user_account`, `jobs`; a module role is still confined to its own schema (D0 preserved); `admin_svc` cannot DDL
- [ ] 3.3 Verify the committed SQL contains no password literals

## 4. Production credential strategy (docs)

- [ ] 4.1 Document the prod secret strategy (secret manager vs IAM database auth) and rotation in `backend/README.md` (or an ops doc), replacing the `*_dev_pw` literals
- [ ] 4.2 Note the dependency/ordering on `add-job-infrastructure` (the `jobs`/`worker_svc` bits) and the per-operator-IAM-role open question

## 5. Validation

- [ ] 5.1 `openspec validate add-ops-db-access --strict` passes
- [ ] 5.2 Bootstrap runs clean on a fresh volume and re-runs clean on an existing one (dev)
