# Contributing

Thanks for your interest in contributing!

## License of contributions (inbound = outbound)
This project is licensed under the **Apache License 2.0**. Per **Section 5**
of that license, any contribution you intentionally submit for inclusion in
the work is provided under the **same Apache 2.0 terms**, with no additional
conditions. By opening a pull request you confirm you have the right to
submit the code under this license.

Apache 2.0 already includes an explicit **patent grant** from contributors,
so a separate CLA is not required for this project. We use the
**Developer Certificate of Origin (DCO)**: sign off each commit with

    git commit -s

which adds a `Signed-off-by:` line certifying you wrote the code or have the
right to submit it.

## Scope
This is a **monorepo**, fully open source (Apache 2.0):
- `apps/*` — Flutter + Rust applications (desktop / Android / iOS);
- `crates/*` — shared pure-Rust libraries (the common engine);
- `packages/*` — shared Dart/Flutter packages;
- `backend/*` — the pure-Rust modular-monolith backend.

Everything is open; there are no closed components. Contributions to any part are welcome.

## File headers
New source files should carry the standard Apache header:

    Copyright 2026 NEETROF
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at
        http://www.apache.org/licenses/LICENSE-2.0

## Brand
Do not add the Cymbra name or logo to forks or derivative distributions —
see [TRADEMARKS.md](TRADEMARKS.md).

## Before submitting
- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- `cargo build --workspace` and `cargo test --workspace` succeed.
- `dart format` is clean and `melos run analyze` reports no issues.
- If you changed the Rust API exposed to Dart, regenerate bindings
  (`cd apps/music && flutter_rust_bridge_codegen generate`) and commit the result.
- Use [Conventional Commits](https://www.conventionalcommits.org/) with a component scope
  (e.g. `feat(music): …`, `fix(engine): …`) — releases are automated from them.
