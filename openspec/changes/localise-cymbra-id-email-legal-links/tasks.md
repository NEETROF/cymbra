# Tasks

## 1. The e-mails (backend/platform)

- [ ] 1.1 `legal_links`: Spanish → `/es/terminos/`, `/es/privacidad/` (D1); tests per locale replacing `non_french_uses_english_legal_links`.
- [ ] 1.2 The layout's `<html lang>` from the locale (D2); test per locale; `emit_samples` covers Spanish.

## 2. Gates and docs

- [ ] 2.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p cymbra-platform -p cymbra-auth` (the crate names as `Cargo.toml` gives them).
- [ ] 2.2 [manual] The owner deploys the site with the Spanish pages, then the backend (D3, M18).
- [ ] 2.3 `openspec validate localise-cymbra-id-email-legal-links --strict` passes, and `python3 scripts/openspec_archive_order.py localise-cymbra-id-email-legal-links` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 32 is marked done in `docs/lingua/language-matrix-programme.md`.
