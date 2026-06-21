<!--
PR title must follow Conventional Commits with a scope, e.g.:
  feat(music): add metronome    fix(engine): correct NoteOff parsing
It becomes the squash-merge commit message that release-please reads.
-->

## Description

<!-- What does this PR do, and why? -->

## Related issues

<!-- e.g. Closes #123 -->

## Type of change

- [ ] feat — new feature
- [ ] fix — bug fix
- [ ] refactor / perf
- [ ] docs
- [ ] chore / build / ci
- [ ] test

## Affected component(s)

- [ ] apps/music
- [ ] crates/* (engine)
- [ ] backend
- [ ] packages/*
- [ ] CI / tooling

## Checklist

- [ ] PR title follows **Conventional Commits** with a scope (e.g. `feat(music): …`)
- [ ] `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass
- [ ] `cargo build --workspace` and `cargo test --workspace` pass
- [ ] `dart format` is clean and `melos run analyze` reports no issues
- [ ] If the Rust↔Dart API changed: regenerated bindings (`cd apps/music && flutter_rust_bridge_codegen generate`) and committed them
- [ ] Docs updated if needed
- [ ] I have the right to submit this under **Apache-2.0** (DCO sign-off: `git commit -s`)
