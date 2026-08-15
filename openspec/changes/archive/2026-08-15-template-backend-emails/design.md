## Context

Backend transactional email today (see `backend/platform/src/email.rs`,
`backend/auth/src/module.rs`, `backend/worker/src/handlers.rs`):

- One port, `EmailSender::send(to, subject, body) -> Result<()>`, with an
  `SmtpSender` (`lettre` async SMTP; Mailpit in dev, a real SMTP provider in prod)
  and a `FakeEmail` test double. `body` is a **single plain-text string** — the
  `lettre::Message` is built with `.body(...)`, not a multipart/HTML alternative.
- Exactly **two** emails, both auth-owned, both built inline with `format!`:
  - Verification — subject `"Verify your Cymbra account"`, body
    `"Confirm your email with this code: {token}"`. Two producers: the sign-up
    path enqueues a `verification_email` **job** (`{to, subject, body}` payload,
    delivered by the worker) so SMTP stays off the request path (design D10); the
    resend path sends **inline**.
  - Password reset — subject `"Reset your Cymbra password"`, body
    `"Reset your password with this code: {tok}"`, sent **inline**.
- The token is a raw UUID printed directly in the body. No HTML, no template
  engine, no branding, no logo, no localization.

The **design system** ("Sonic Luminescence") is a single shared identity:
`apps/music/lib/theme/cymbra_theme.dart` (`CymbraColors`) is the source of truth
and `apps/back-office/src/styles.css` mirrors it 1:1. Palette: surface navy
`#0B1326` / panel `#131B2E`, text `#DAE2FD` / muted `#9AA1BA`, primary lilac
`#D2BBFF` and violet `#7C3AED`, teal `#44E2CD`, green `#4EDEA3`, amber `#FFB454`,
error coral `#FFB4AB`. Font: `system-ui, -apple-system, "Segoe UI", Roboto,
sans-serif` (no custom brand webfont). The back office renders its wordmark as a
**gradient text "C" mark** (`linear-gradient(145deg, #7C3AED, #B58BFF)`), not an
image. Product name **Cymbra**; copyright holder **NEETROF**. Apps are localized:
Flutter en/es/fr/it, back office en/fr.

Crucially, these emails belong to the **shared identity system**, not to Music:
auth serves multiple audiences (`CYMBRA_ALLOWED_AUDIENCES=music,live`, growing),
and the existing subjects are already umbrella ("your Cymbra account"). There is
**no product-neutral Cymbra logo asset** in the repo today — only Music's icons,
and the back office uses a CSS text mark, not an image.

Constraint: CLAUDE.md requires Rust line coverage ≥ 80% and `email.rs` is **not**
in the coverage-exclusion regex, so rendering logic must be pure and tested.

## Goals / Non-Goals

**Goals:**
- Both existing emails render as branded HTML matching the Cymbra palette, with a
  plain-text alternative, from one shared, reusable, host-testable render layer.
- Identical output regardless of producer (inline send vs. `verification_email`
  job).
- Localized subject + body (en/es/fr/it) with a guaranteed English fallback.
- Email-client-safe output (table layout, inline styles, dark-mode resilient),
  robust when images are blocked (no reliance on remote images).
- No public gRPC/API break; the added locale field is optional and additive.

**Non-Goals:**
- No new email *types* (welcome, receipts, moderation notifications) — only the
  two that exist. The layer is built to make adding more trivial, but none are
  added here.
- No change from the code-entry (OTP) flow to click-through verification links
  (no app deep links / web verify page, no `base_url` config) — the code stays,
  shown in a styled box; the layout only *reserves* a CTA slot for later (D9).
- No switch of email provider or transport — still `lettre`/SMTP.
- No marketing email, unsubscribe management, or a full localization framework
  beyond these two messages' strings.
- No per-product email skinning — one shared "Cymbra ID" brand (D5, D10).
- No reuse of the Music app's logo/branding assets for these emails.

## Decisions

### D1 — A shared, pure `email_template` module in `cymbra-platform`
Rendering lives next to `EmailSender` in the platform crate so **both** the auth
inline senders and the worker/job producer depend on it without a new dependency
edge. It exposes typed constructors returning a rendered message, e.g.
`verification_email(code, locale) -> RenderedEmail` and
`password_reset_email(code, locale) -> RenderedEmail`, where
`RenderedEmail { subject: String, html: String, text: String }`.
*Alternative considered:* render in the worker from a `template_id + context`
payload. Rejected because the resend/reset paths send inline (never touch the
worker), so the worker cannot be the single render site — a shared library is the
only place both paths meet.

### D2 — Producer renders; the job carries rendered `{subject, html, text}`
The `verification_email` job payload becomes the rendered message (plus `to`),
not a template id. Keeps the worker a dumb transport, keeps all content/branding
in one crate, and preserves idempotency (a single-use token means a re-delivered
job simply re-sends identical rendered mail).
*Trade-off:* larger job rows and a template change won't restyle already-enqueued
jobs — acceptable, since queued verification jobs drain in seconds.

### D3 — `askama` for compile-time HTML templates
Add `askama` (typed, compile-time-checked templates, no runtime files to ship in
the image) for the base layout + per-email bodies. Locale strings are provided to
the template as typed context.
*Alternatives:* `maud` (macro HTML — less readable for table-heavy email markup);
`tera`/`handlebars` (runtime parsing + shipping template files, weaker typing);
MJML (best-in-class email HTML but a Node build step — wrong toolchain for a Rust
service). Askama gives type safety and a single self-contained binary.

### D4 — One brand layout, table-based, inline styles, no remote assets
A single base layout wraps every email: navy `#0B1326` page, centered `#131B2E`
card (max-width ~600px), header logo/wordmark (D5), content slot with an optional
CTA-button slot (D9), footer (NEETROF + legal links). Structured with `<table>`
and **inline** `style` attributes (email
clients strip `<style>`/`<head>`, Flexbox/Grid, and webfonts). Uses the system
font stack. The code is shown in a large, letter-spaced, high-contrast box.

### D5 — Shared-identity ("Cymbra ID") branding, not per-product; neutral hosted logo + text fallback
These emails come from the **shared identity hub** (`cymbra-auth`/`cymbra-platform`),
which serves every audience (`CYMBRA_ALLOWED_AUDIENCES=music,live`, more to come),
so they carry the **umbrella "Cymbra ID" brand — never a single product's**. The
Music app icon/splash (`apps/music/assets/branding/*`) MUST NOT be used; the copy
is already umbrella ("your Cymbra account").
- **Brand name** (header wordmark + `From` display name): **"Cymbra ID"**.
  `From: "Cymbra ID <no-reply@cymbra.app>"`.
- **Header logo:** the Cymbra app icon **reduced to the "C" mark, with the
  audio/music motif removed** (product-neutral identity glyph), served as a hosted
  `<img alt="Cymbra ID">` **from the `cymbra.app` marketing site**, layered over a
  solid-color **"Cymbra ID" text wordmark** so blocked images degrade to visible
  text, never to nothing.
- That neutral "C" asset **does not exist yet** — a design dependency (see Risks).
  To avoid blocking implementation, the layer ships with the **text wordmark
  first** and the `<img>` (its `cymbra.app` URL, config-driven) is added once the
  asset lands; the layout reserves its slot.
*Explicitly rejected:* (a) reusing the Music PNG — wrong brand for shared
identity; (b) a CSS **gradient-clipped** text wordmark
(`background-clip: text` + transparent color) like the back office's mark — not
email-safe (Outlook/Gmail render it invisible); (c) CID-embedded image (bloat,
Outlook quirks). The **code is never an image** — always live text in a styled
box, so it survives image blocking regardless.

### D10 — Umbrella brand by default, not per-audience skinning
The template does **not** re-skin per originating product (e.g. "Sign in to
Cymbra Live"). Per-audience skinning is rejected: it contradicts a *shared*
identity, would force threading `audience` into the verify/reset flows (which
don't carry it today), and the copy is already umbrella. If a future module needs
a distinct brand, the render layer can key off `audience` then — but the default
is one Cymbra ID identity brand.

### D6 — Multipart `EmailSender`; struct payload
Change the port to carry both representations, e.g.
`send(&self, to: &str, email: &RenderedEmail) -> Result<()>`, and build a
`lettre` **multipart/alternative** message (plain text + HTML) so every client —
including text-only and screen readers — gets a usable body and code.
`FakeEmail` records `(to, subject, html, text)` for assertions.

### D7 — Locale threading: additive, optional, English-default
The template layer is locale-parameterized. The client (which already knows its
locale) passes it via a new **optional** `locale` field on the affected auth
requests (sign-up, resend-verification, request-password-reset). Backend maps it
to the nearest supported language and **falls back to English** when absent or
unrecognized — so the branding ships value even before every client wires the
field. Localized strings live in the `email_template` module (typed per-locale
tables), not a new i18n framework.

### D9 — Keep the OTP code-entry flow; reserve a CTA slot
The emails keep presenting a code the user types in-app, rather than a
click-through verification/reset link. This matches the shipped product: the app
is already built around code entry (`otp_verify_screen.dart` — "Email
verification by code" — and `forgot_password_screen.dart`), and there is **no
deep-link infrastructure** (iOS `Info.plist` declares only the Google OAuth URL
scheme, no `associated-domains`; Android declares only `LAUNCHER` + USB
intent-filters, no `http`/`autoVerify` app link; no `app_links`/`go_router`).
A link would also be **fragile**: email security scanners (e.g. Microsoft Safe
Links) prefetch URLs and can **consume a single-use token before the user
clicks**, breaking the flow — codes are immune and work cross-device. The layout
nonetheless reserves a **CTA-button slot** so a future "verify-by-link" change
(which must add associated-domains/app-links + app routes) can drop in without a
redesign.
*Alternative considered:* convert to click-through links now — rejected as a
separate, much larger feature disproportionate to templating the emails.

### D8 — Testing: pure renderers + invariant/snapshot assertions
`RenderedEmail` constructors are pure and unit-tested: each asserts the code is
present in **both** html and text, the subject is the localized one, the brand
signature is present (accent `#7C3AED`, wordmark "Cymbra", NEETROF footer), and
that unknown locales fall back to English. The SMTP multipart glue stays thin;
`FakeEmail` covers the send seam in higher-level tests. Keeps `email.rs`/template
code ≥ 80% without new mocking infrastructure.

## Risks / Trade-offs

- **Dark-mode / client CSS mangling (Gmail, Outlook, Apple Mail invert or
  restyle colors).** → Commit to a dark design with explicit `background-color`
  on every cell, avoid relying on default text color, keep contrast high, and
  test the rendered HTML against a preview (Mailpit in dev) before shipping.
- **Outlook (Word engine) ignores much modern CSS.** → Table layout + inline
  styles only; no divs-for-layout, no shorthand that Outlook drops; VML not
  needed since there's no background image.
- **Job payload shape change is breaking for in-flight `verification_email`
  jobs across deploy.** → Deploy worker (tolerant reader) before/with producer;
  drain or accept that any pre-deploy queued job (single-use token, seconds-lived)
  may fail once and DLQ — negligible volume.
- **Locale field unwired in some clients.** → English fallback guarantees a valid
  branded email regardless; wiring clients is incremental.
- **Coverage regression if HTML strings live in `email.rs`.** → Keep rendering in
  a dedicated, fully-tested module; keep transport glue minimal.
- **The neutral "Cymbra ID" logo asset does not exist yet (design dependency).**
  → Ship the text-wordmark header first (implementation isn't blocked); add the
  hosted `<img>` once the neutral asset is designed and hosted.

## Migration Plan

1. Add `askama` to `backend/platform`; add the `email_template` module (layout,
   two renderers, locale tables) with tests — no behavior change yet.
2. Change `EmailSender`/`RenderedEmail`/`SmtpSender`/`FakeEmail` to multipart;
   update all call sites to render then send. Update the `verification_email`
   job payload + worker handler together.
3. Add the optional `locale` field to the auth proto + regenerate clients; pass
   locale from the Flutter app and back office; default English server-side.
4. Verify end-to-end against Mailpit (visual check of both emails, all locales),
   then deploy **worker first** (tolerant to old+new payloads), then server.

*Rollback:* the change is additive at the API layer (optional field) and internal
elsewhere; revert the crate + regenerate. No data migration to undo.

## Resolved Decisions

The earlier open questions are now settled:

- **Q1 — Localization:** translate both emails into **all four app locales**
  (en/es/fr/it) now, with English fallback (D7).
- **Q2 — Sender:** `From: "Cymbra ID <no-reply@cymbra.app>"` via
  `CYMBRA_SMTP_FROM`. *Action item (prod, not a code task):* confirm
  **SPF/DKIM/DMARC** for `cymbra.app` before prod so branded HTML mail authenticates
  and doesn't land in spam / read as phishing.
- **Q3 — Footer legal links:** reuse the **`cymbra.app` legal pages**, locale-aware
  per the `legal-links` spec — FR → `https://cymbra.app/cgu/` +
  `https://cymbra.app/confidentialite/`; every other locale (incl. es/it) →
  `https://cymbra.app/en/terms/` + `https://cymbra.app/en/privacy/`. Footer carries
  **Terms + Privacy** (transactional mail needs no unsubscribe). The tiny resolver
  is ported into `email_template`, keyed on the email locale.
- **Q4 — Logo asset:** the neutral mark is the **Cymbra app icon reduced to the
  "C", audio motif removed**. *External dependency:* this asset must be produced;
  the header ships as the "Cymbra ID" text wordmark until it lands (D5).
- **Q5 — Logo hosting:** served from the **`cymbra.app` marketing site**; the email
  references its URL (config-driven, `CYMBRA_EMAIL_LOGO_URL`).
- **Q6 — Subjects:** **unchanged** ("Verify your Cymbra account" / "Reset your
  Cymbra password") — already umbrella; "Cymbra ID" is carried by the wordmark +
  `From` name.

## Open Questions

- Footer: confirm no link beyond Terms + Privacy is wanted (e.g. a help/contact URL).
- Timing/owner for producing the neutral "C" asset and adding it to `cymbra.app`.
