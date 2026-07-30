## Why

The backend sends transactional email (account verification, password reset) as
bare plain-text bodies built inline with `format!` — no branding, no HTML, no
localization, just a raw UUID code (e.g. `"Confirm your email with this code: <uuid>"`).
These are the first impression a new user gets and the only touchpoint during a
password reset, yet they look nothing like the Cymbra apps, undermine trust
(unbranded mail reads as phishing, especially one containing a code), and ship
English-only to a product that is localized in four languages and has a
French-first audience. Templating them to the shared "Sonic Luminescence" design
system used by both Cymbra Music (Flutter) and Cymbra Back Office (Vue) fixes all
of this in one place.

## What Changes

- Introduce a **shared email-template layer** in the backend that renders each
  transactional email as **branded, email-client-safe HTML with a plain-text
  alternative**, from typed inputs (recipient locale + the code). Rendering lives
  in one reusable module so both the inline senders (`resend_verification`,
  `request_password_reset`) and the `verification_email` job produce identical output.
- Add a **single brand layout** (header wordmark, content slot, footer with the
  NEETROF attribution and legal links) using the Cymbra palette — navy `#0B1326`
  surface, violet `#7C3AED` / lilac `#D2BBFF` accents, `#DAE2FD` text — with the
  code shown in a prominent styled box. Table-based, inline-styled, dark-mode-safe.
- Brand the emails with the **shared "Cymbra ID" identity brand**, not any single
  product (they come from the multi-audience identity system). No reuse of Cymbra
  Music's logo; a product-neutral Cymbra ID logo is a separate design dependency,
  so the header ships as an email-safe **"Cymbra ID" text wordmark** first.
- **Extend the `EmailSender` port to carry multipart HTML + text** (currently a
  single plain-text `body` arg). **BREAKING** to the internal trait signature and
  the `verification_email` job payload — no public/gRPC contract changes.
- **Localize** the two emails (subject + body) to the flagship app's locales
  (en, es, fr, it), defaulting to English. Thread the requester's locale from the
  client through the auth RPCs as an **additive, optional** field (backwards
  compatible; absent = English).
- Snapshot/invariant tests over the rendered output; keep pure rendering
  host-testable and ≥ 80% covered.

## Capabilities

### New Capabilities
- `transactional-email`: branded, localized rendering and multipart delivery of
  backend-sent transactional email (shared brand layout, HTML + plain-text
  alternative, locale selection with English fallback), applied to the existing
  account-verification and password-reset messages.

### Modified Capabilities
<!-- None. backend-auth still sends a verification/reset email on the same triggers;
     only the presentation/format changes, which is the new capability's concern. -->

## Impact

- **Code**:
  - `backend/platform/src/email.rs` — `EmailSender` trait signature (multipart),
    `SmtpSender` (multipart message), `FakeEmail` (record html+text).
  - `backend/platform/` — new `email_template` module (layout + per-email
    renderers + locale strings); new `askama` (compile-time templates) dependency.
  - `backend/auth/src/module.rs` — build rendered emails via the template layer;
    `verification_email_job` payload gains html/text (or template id + context);
    thread `locale` into `resend_verification` / `request_password_reset` / sign-up.
  - `backend/worker/src/handlers.rs` — `EmailJob` payload + send call updated for
    multipart.
  - `backend/auth-port/proto/auth.proto` (+ generated clients) — optional `locale`
    field on the affected requests; app/back-office clients pass their locale.
- **Config**: optional sender display name; no new required env vars.
- **Tests**: `FakeEmail` assertions updated to the new shape; new render tests.
- **No** DB schema, no new external service (still SMTP via `lettre`).
