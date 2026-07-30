## 1. Render layer (cymbra-platform)

- [x] 1.1 Add `askama` to `backend/platform/Cargo.toml` (workspace-pinned) and wire its build config.
- [x] 1.2 Create `backend/platform/src/email_template/` with a `RenderedEmail { subject, html, text }` type and a base brand layout (`layout.html`): navy `#0B1326` page, `#131B2E` card (~600px), header, content slot + optional CTA-button slot, footer with NEETROF attribution + legal links. Table-based, inline styles only. Shared-identity brand only — do NOT reference `apps/music` assets.
- [x] 1.3 Header brand: solid-color **"Cymbra ID"** text wordmark (email-safe; NO gradient-clipped text; NO Music logo). Reserve the `<img>` slot for the hosted neutral logo, its URL from config (`CYMBRA_EMAIL_LOGO_URL`).
- [ ] 1.3a (external design/site dependency, non-blocking) Produce the neutral **Cymbra ID** logo = the Music app icon reduced to the **"C"** with the audio motif removed; add it to the **`cymbra.app`** site at a stable URL; then wire `<img alt="Cymbra ID">` above the text wordmark fallback. Until then the wordmark ships alone.
- [x] 1.4 Add locale support: a `SupportedLocale` enum (en/es/fr/it) with a parser that maps an optional/arbitrary locale string to a supported locale, defaulting to English.
- [x] 1.5 Footer legal links, locale-aware (port from the `legal-links` spec): fr → `https://cymbra.app/cgu/` + `https://cymbra.app/confidentialite/`; every other locale (incl. es/it) → `https://cymbra.app/en/terms/` + `https://cymbra.app/en/privacy/`. Footer shows Terms + Privacy + NEETROF attribution.
- [x] 1.6 Implement `verification_email(code, locale) -> RenderedEmail` (subject + body + code box) with the localized strings for en/es/fr/it.
- [x] 1.7 Implement `password_reset_email(code, locale) -> RenderedEmail` with the localized strings for en/es/fr/it.
- [x] 1.8 Ensure the plain-text alternative for each email carries the same code and instructions as the HTML.

## 2. Email port → multipart (cymbra-platform)

- [x] 2.1 Change `EmailSender::send` to carry the rendered message (e.g. `send(&self, to: &str, email: &RenderedEmail)`), updating the trait in `backend/platform/src/email.rs`.
- [x] 2.2 Update `SmtpSender::send` to build a `lettre` multipart/alternative message (plain-text + HTML) from `RenderedEmail`. Set `From` to `"Cymbra ID <no-reply@cymbra.app>"` via `CYMBRA_SMTP_FROM` (parsed as a display-name `Mailbox`); update `.env.example` + `deploy/.env.prod.example` + `CYMBRA_EMAIL_LOGO_URL`.
- [x] 2.3 Update `FakeEmail` to record `(to, subject, html, text)` and expose accessors for test assertions.

## 3. Producers (cymbra-auth + worker)

- [x] 3.1 In `backend/auth/src/module.rs`, thread the recipient locale into `sign_up_local`, `resend_verification`, and `request_password_reset` (optional, default English).
- [x] 3.2 Replace the inline `format!` bodies in `resend_verification` and `request_password_reset` with `email_template` renders + the new multipart `send`.
- [x] 3.3 Update `verification_email_job` to carry the rendered `{to, subject, html, text}` (+ locale used) as the job payload.
- [x] 3.4 Update the worker `EmailJob` payload struct and the `verification_email` handler in `backend/worker/src/handlers.rs` to deserialize and send the multipart message.

## 4. Locale plumbing (proto + clients)

- [x] 4.1 Add an optional `locale` field to the affected requests in `backend/auth-port/proto/auth.proto` (sign-up, resend-verification, request-password-reset); regenerate Rust + Dart/TS clients.
- [x] 4.2 Map the proto `locale` to a `SupportedLocale` in the auth gRPC layer, defaulting to English when absent.
- [x] 4.3 Pass the app locale from the Flutter client (`apps/music`) on the affected auth calls.
- [x] 4.4 N/A — the back office has no local sign-up / resend / password-reset surface (moderators authenticate via OIDC), so there is no client call to thread `locale` through.

## 5. Tests

- [x] 5.1 Unit-test each renderer: code present in both html and text, correct localized subject, brand signature present (`#7C3AED`, "Cymbra ID" wordmark, NEETROF footer), code box present, no reference to any Music/product asset.
- [x] 5.1a Test footer legal links resolve per locale (fr → cgu/confidentialite; es/it/en → en terms/privacy).
- [x] 5.2 Test locale selection: each supported locale renders its language; unknown/missing locale falls back to English while staying fully branded.
- [x] 5.3 Test that the job-produced and inline-produced verification emails are identical for the same code + locale.
- [x] 5.4 Update existing `FakeEmail`-based auth/worker tests to the new multipart shape and assert the branded output.
- [x] 5.5 Confirm Rust line coverage stays ≥ 80% (`cargo llvm-cov --workspace --fail-under-lines 80`).

## 6. Verify & finalize

- [x] 6.1 Send both emails to Mailpit in all four locales; visually verify branding, dark-mode legibility, and code prominence in the HTML preview and the text fallback.
- [x] 6.2 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`; `melos run analyze` clean for the client changes.
- [x] 6.3 `openspec validate template-backend-emails --strict` passes.
- [ ] 6.4 (prod ops, out-of-code) Confirm **SPF/DKIM/DMARC** for `cymbra.app` and that `no-reply@cymbra.app` is an authorized sender before the prod rollout.
