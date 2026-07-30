## Why

The "set a password on a Google/Apple account" flow (`SetLocalCredential`, from
`add-account-identity-linking`) binds the email/`local` identity to the account
**before** the emailed code is verified: `set_local_credential` does
`creds.insert(email, hash)` + `link_identity(user_id, "local", email)` and only
then sends the verification email. No account takeover is possible (an unverified
credential can't sign in — `FailedPrecondition "email not verified"`), **but** the
arbitrary email is reserved with no proof of ownership:

- `local_credentials.email` and `user_identities (provider, subject)` are unique,
  so anyone else — including the real owner — now gets `AlreadyExists`.
- Nothing cleans it up: the orphan reaper only deletes handle-less accounts
  (`WHERE handle IS NULL`), and a Google/Apple account has a handle, so the
  unverified credential (and the squat) persists indefinitely past the 24h token.
- An unsolicited verification email is sent to the target (spam vector).

Net: a signed-in user can **squat any email** and deny it to its rightful owner,
without ever proving they control it.

## What Changes

- **Defer the binding until verification.** On `SetLocalCredential`, validate for
  UX (reject a weak password; reject if the account already has a `local` identity)
  and store a **pending set-password** — `{user_id, email, argon2 password hash}` —
  keyed by the freshly minted verification token with a TTL of `verify_ttl` (24h).
  Send the (unchanged) branded/localized verification email. **Do not** touch
  `local_credentials` or `user_identities` yet, so the email is **not reserved**.
- **Bind on verification.** When `VerifyEmail` receives a token matching a pending
  set-password, atomically insert the `local_credential` (already **verified**) +
  `link_identity(user_id, "local", email)` + clear the pending record. If the email
  was taken meanwhile, fail cleanly with `AlreadyExists` (first-verify-wins; the
  loser never reserved anything).
- **Self-cleaning, no reaper**: the pending record lives in the cache with a TTL,
  so an abandoned set-password simply expires — no squat, nothing to reap.
- Keep the current UX (the "check your email" message + the existing OTP screen via
  the sign-in `FailedPrecondition` path) and the 3/hour send rate limit.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `account-linking`: the **"Link a local (email + password) credential"**
  requirement changes — the account gains the `local` identity only **after** the
  emailed code is verified (not on submit), and the email is not reserved before
  then.

## Impact

- **Code (backend only)**:
  - `backend/auth/src/module.rs` — `set_local_credential` (validate + store pending,
    no insert/link); `verify_email` (consume a pending set-password → create verified
    credential + link).
  - A small **pending-set-password store** seam (cache-backed via
    `cymbra_platform::cache`), doubled with a mock/hand fake per the `rust-testing` skill.
  - Serialization of the pending record (`{user_id, email, password_hash}`) —
    argon2 hash only, never plaintext.
- **No DB migration**, no proto/API change (`VerifyEmail`/`SetLocalCredential`
  signatures unchanged), no client/Flutter change.
- **Depends on / builds on** the existing branded, localized verification email
  (`template-backend-emails`) — email content is unchanged.
- **Security**: removes the email-squatting/DoS + eliminates the unbounded
  unverified-credential residue; preserves the no-takeover property.
