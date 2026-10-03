## 1. Pack set (lingua-core)

- [ ] 1.1 `crates/lingua-core/src/packs/set.rs`: `PackSet` with `new(pack)`, `add(pack)` (refuses a language already held), `resolve(Option<&str>)` (`None` gives the default; an unknown tag, or a language not held, gives an error naming the tag), `default_language()` and `languages()` (default first), and `PackSetError` with `Display`. Unit tests cover each branch, including that a refused `add` leaves the set as it was. Run `cargo test -p lingua-core packs::set`.

## 2. Engine (lingua-wasm)

- [ ] 2.1 The engine holds a `PackSet`. Add `addPack(bytes)` (malformed, incompatible or duplicate → `JsError`) returning the tag, and `languages()` returning the held tags as JSON. Level vocabularies are cached per language. `const EN` is deleted from `src/lib.rs`.
- [ ] 2.2 The 21 language-bound bindings of design D2 take a trailing `language: Option<String>`, resolve it before touching state, and return `Result<_, JsError>`. The root bindings of D3 keep their signatures.
- [ ] 2.3 Exports name each record's language. Applies read each record's language (missing means `en`) and skip the languages without a pack, without counting them. `restore` prunes each language that has a pack against its own pack (D5).
- [ ] 2.4 Native tests (`crates/lingua-wasm/tests/languages.rs`), on the testdata en-fr pack plus a Spanish pack built in-test with `lingua-pack`. Run `cargo test -p lingua-wasm`. Cases:
  - analysis in each language and with no language;
  - `addPack` of a second English pack is refused;
  - a Spanish status exports `es`;
  - a Spanish change applied to an English-only engine changes nothing, and on the two-pack engine it is recorded;
  - `hasLevels`, `notice` and the ladder answer per pack.
- [ ] 2.5 A wasm-bindgen test (node): a binding naming a language the engine holds no pack for throws, and so does `addPack` of a duplicate. Run `wasm-pack test --node crates/lingua-wasm`.

## 3. English invariance and the extension

- [ ] 3.1 The S0 harness unwraps the new results. `cargo test -p lingua-wasm --test english_baseline` passes, and `git diff --stat origin/main -- crates/lingua-wasm/tests/baseline` is empty.
- [ ] 3.2 `yarn gen:wasm`: the generated `lingua_wasm.d.ts` types every new `language` as optional. `yarn typecheck`, `yarn test` and `yarn build` pass, and `git diff --stat origin/main -- apps/lingua-extension/src` is empty.

## 4. Gates

- [ ] 4.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`.
- [ ] 4.2 `openspec validate generalise-lingua-wasm-engine --strict`; `python3 scripts/check_ci_units.py --list` is unchanged; in `docs/lingua/spanish-programme.md`, change 6 is marked done and change 7 next.
