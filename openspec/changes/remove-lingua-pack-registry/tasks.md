## 1. Back office

- [ ] 1.1 Draw the studied-language filter's options from the usage report's per-language breakdown, plus the selected language, in `apps/back-office/src/views/LinguaView.vue`; unit-test that a window without the selected language keeps it offered and selected
- [ ] 1.2 Remove the "Packs de données" section from `LinguaView.vue`, and the `loadPacks` call on mount
- [ ] 1.3 Remove `DataPack`, `packs` and `loadPacks` from `apps/back-office/src/stores/lingua.ts`
- [ ] 1.4 Remove `adminListDataPacks` and `linguaPacks` from `apps/back-office/src/lib/e2e-seam.ts`
- [ ] 1.5 Remove the section's keys from `apps/back-office/src/i18n/locales/en.json` and `fr.json`, keeping the two files aligned
- [ ] 1.6 In `apps/back-office/e2e/lingua.spec.ts`, replace the pack-row assertion with one on the studied-language filter, drawn from the breakdown

## 2. Backend

- [ ] 2.1 Remove `rpc AdminListDataPacks`, `AdminListDataPacksRequest`, `AdminListDataPacksResponse` and `LinguaDataPack` from `backend/lingua/proto/lingua_admin.proto`, and the comment that introduces them
- [ ] 2.2 Remove `admin_list_data_packs` and `pack_to_proto` from `backend/lingua/src/admin_grpc.rs`, with their tests
- [ ] 2.3 Remove the registry from `LinguaAdminModule` in `backend/lingua/src/admin.rs` — the manifest parse at construction, `list_packs`, and `list_packs_reads_the_embedded_registry` — so constructing the module can no longer fail on a manifest
- [ ] 2.4 Delete `backend/lingua/src/pack_registry.rs` and `backend/lingua/packs-manifest.json`, and the module declaration
- [ ] 2.5 Regenerate the back office's and any other client's stubs from the changed `.proto`, and confirm nothing else referenced the removed messages

## 3. Pack pipeline

- [ ] 3.1 Delete `crates/lingua-pack/src/manifest.rs` and `crates/lingua-pack/src/bin/lingua-pack-manifest.rs`, and their module declaration
- [ ] 3.2 Remove `the_committed_pack_manifest_matches_a_fresh_build` from `crates/lingua-pack/src/lib.rs`
- [ ] 3.3 Remove the `emit-manifest` and `check-manifest` modes from `scripts/lingua-data/build.sh`, and the header paragraph describing the registry
- [ ] 3.4 Remove `/bin/lingua-pack-manifest\.rs` from `.github/coverage-ignore-regex.txt`
- [ ] 3.5 Grep the repository for `packs-manifest`, `emit-manifest`, `check-manifest`, `pack_registry` and `AdminListDataPacks` outside `openspec/changes/archive/`, and leave none but this change's own artefacts

## 4. Gates

- [ ] 4.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 4.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the shared ignore regex
- [ ] 4.3 Back office: lint, typecheck, unit tests and the Playwright e2e suite
- [ ] 4.4 The pull request title carries the breaking marker, and the `proto` workflow reports the removed RPC without failing
- [ ] 4.5 `openspec validate remove-lingua-pack-registry --strict` passes
