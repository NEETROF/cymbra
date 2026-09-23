## 1. Back office

- [x] 1.1 Draw the studied-language filter's options from the usage report's per-language breakdown, plus the selected language, in `apps/back-office/src/views/LinguaView.vue`; unit-test that a window without the selected language keeps it offered and selected
- [x] 1.2 Remove the "Packs de données" section from `LinguaView.vue`, and the `loadPacks` call on mount
- [x] 1.3 Remove `DataPack`, `packs` and `loadPacks` from `apps/back-office/src/stores/lingua.ts`
- [x] 1.4 Remove `adminListDataPacks` and `linguaPacks` from `apps/back-office/src/lib/e2e-seam.ts`
- [x] 1.5 Remove the section's keys from `apps/back-office/src/i18n/locales/en.json` and `fr.json`, keeping the two files aligned
- [x] 1.6 In `apps/back-office/e2e/lingua.spec.ts`, replace the pack-row assertion with one on the studied-language filter, drawn from the breakdown

## 2. Backend

- [x] 2.1 Remove `rpc AdminListDataPacks`, `AdminListDataPacksRequest`, `AdminListDataPacksResponse` and `LinguaDataPack` from `backend/lingua/proto/lingua_admin.proto`, and the comment that introduces them
- [x] 2.2 Remove `admin_list_data_packs` and `pack_to_proto` from `backend/lingua/src/admin_grpc.rs`, with their tests
- [x] 2.3 Remove the registry from `LinguaAdminModule` in `backend/lingua/src/admin.rs` — the manifest parse at construction, `list_packs`, and `list_packs_reads_the_embedded_registry` — so constructing the module can no longer fail on a manifest
- [x] 2.4 Delete `backend/lingua/src/pack_registry.rs` and `backend/lingua/packs-manifest.json`, and the module declaration
- [x] 2.5 Regenerate the back office's and any other client's stubs from the changed `.proto`, and confirm nothing else referenced the removed messages

## 3. Pack pipeline

- [x] 3.1 Delete `crates/lingua-pack/src/manifest.rs` and `crates/lingua-pack/src/bin/lingua-pack-manifest.rs`, and their module declaration
- [x] 3.2 Remove `the_committed_pack_manifest_matches_a_fresh_build` from `crates/lingua-pack/src/lib.rs`
- [x] 3.3 Remove the `emit-manifest` and `check-manifest` modes from `scripts/lingua-data/build.sh`, and the header paragraph describing the registry
- [x] 3.4 Remove `/bin/lingua-pack-manifest\.rs` from `.github/coverage-ignore-regex.txt`
- [x] 3.5 Grep the repository for `packs-manifest`, `emit-manifest`, `check-manifest`, `pack_registry` and `AdminListDataPacks` outside `openspec/changes/archive/`, and leave none in code — the only mentions left are this change's artefacts and `add-lingua-expression-table`'s, a shipped change's record that moves to the archive as it stands

## 4. Gates

- [x] 4.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 4.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the shared ignore regex
- [x] 4.3 Back office: lint, typecheck, unit tests and the Playwright e2e suite
- [x] 4.4 The pull request title carries the breaking marker, and the `proto` workflow reports the removed RPC without failing
- [x] 4.5 `openspec validate remove-lingua-pack-registry --strict` passes
