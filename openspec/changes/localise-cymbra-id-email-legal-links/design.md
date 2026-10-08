# Design — localise-cymbra-id-email-legal-links

## Context

See proposal.md (Why). `backend/platform/src/email_template/mod.rs`: `SupportedLocale {En, Es, Fr,
It}` parsed from the primary subtag, English otherwise; `legal_links(locale)` French or English;
`copy_for` per locale, `shared_copy` the footer labels (es « Términos »/« Privacidad »); one Askama
layout `templates/email/layout.html`, `<html lang="en">` hard-coded, footer `{{ terms_url }}`
`{{ privacy_url }}`; the text part repeats both URLs. Tests: `french_localizes_subject_and_legal_links`,
`non_french_uses_english_legal_links` (asserts es and it get `/en/`), `each_locale_renders_its_language`,
the ignored `emit_samples`; `backend/auth/src/module.rs` asserts `/cgu/` for a French sign-up. The
sign-up e-mail is rendered when the job is enqueued; the worker only sends it.

## Decisions

### D1 — Links per locale, a Spanish site page or English

`legal_links` answers the Spanish pages for `Es`; `En` and `It` keep the English ones. The paths are
constants beside the French ones; a test pins each locale's pair.

### D2 — The layout's `lang`

The layout takes `lang` (`fr`, `en`, `es`, `it`) from the locale. A test asserts it per locale.

### D3 — Deploy order

The site with the Spanish pages is deployed before the backend (owner). The backend's deploy is
manual (`backend-deploy`); the tasks name the order.

## Risks / Trade-offs

- **A Spanish link before the page is live** → D3.
- **Music's Spanish users see two rules** (Spanish pages in e-mails, English in the app) → said in the
  proposal; the app's rule is Music's to change.

## Migration Plan

Deploy the site (change 29), then the backend. Queued e-mails keep their links.
