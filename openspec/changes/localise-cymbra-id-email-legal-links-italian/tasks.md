# Tasks

## 1. The e-mails (backend/platform)

- [ ] 1.1 `legal_links`: the `En | It` arm split, Italian → `https://cymbra.app/it/termini/` and `https://cymbra.app/it/privacy/` (the paths change 41 pinned); its doc comment names the four languages; `italian_uses_english_legal_links` becomes `italian_uses_italian_legal_links`, both `/it/` addresses in both parts and no `https://cymbra.app/en/` (D1).

## 2. Gates and deploy

- [ ] 2.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p cymbra-platform`.
- [ ] 2.2 [owner] The implementation pull request merges only after change 41's site is deployed (itself after the backend carrying `thanks_path`); then deploy the backend (D2).
- [ ] 2.3 `openspec validate localise-cymbra-id-email-legal-links-italian --strict`; `python3 scripts/openspec_archive_order.py localise-cymbra-id-email-legal-links-italian` (exit 10 naming exactly the two changes of `.openspec.yaml`, 0 once they are archived); `docs/lingua/italian-programme.md` row 43's status.
