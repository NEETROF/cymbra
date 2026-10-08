# Tasks

## 1. The e-mails (backend/platform)

- [ ] 1.1 `legal_links`: Spanish → `/es/terminos/`, `/es/privacidad/`, and its doc comment (D1); tests per locale replacing `non_french_uses_english_legal_links`.
- [ ] 1.2 The layout's `<html lang>` from the resolved locale (D2); a test per locale and one for `de` → `lang="en"`.
- [ ] 1.3 `apps/site/src/lib/pinned-routes.ts` gains `/es/terminos/` and `/es/privacidad/`, `pinnedBy` « Cymbra ID e-mail footer, es », source `backend/platform/src/email_template/mod.rs`.

## 2. Gates and docs

- [ ] 2.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p cymbra-platform -p cymbra-auth` (the crate names as `Cargo.toml` gives them).
- [ ] 2.2 [manual] The owner deploys the site with the Spanish pages, then the backend (D3, M18).
- [ ] 2.3 `openspec validate localise-cymbra-id-email-legal-links --strict` passes, and `python3 scripts/openspec_archive_order.py localise-cymbra-id-email-legal-links` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 32 is marked done in `docs/lingua/language-matrix-programme.md`.
