# Cymbra

Open-source suite around an interactive music engine — Flutter (UI) + Rust (engine),
bridged with [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge).
Free and open source under the [Apache License 2.0](LICENSE).

> Brand notice: the code is open; the name **"Cymbra"** and its logo are trademarks of
> **NEETROF** and are not licensed (Apache 2.0 §6). See [TRADEMARKS.md](TRADEMARKS.md).

## Monorepo layout

```
apps/        # deployable Flutter + Rust apps (desktop / Android / iOS)
  music/     # interactive piano (the first app; was the POC)
crates/      # shared pure-Rust libraries (the common engine) — coming soon
packages/    # shared Dart/Flutter packages (UI kit, theme) — coming soon
backend/     # pure-Rust modular-monolith backend — coming soon
```

The taxonomy is **deployable vs shared**, per ecosystem:

|        | Deployable | Shared library |
|--------|------------|----------------|
| Rust   | `backend/` | `crates/`      |
| Dart   | `apps/`    | `packages/`    |

## Tooling

- **Rust**: a single Cargo workspace (root `Cargo.toml`), edition 2024.
- **Dart/Flutter**: [Melos](https://melos.invertase.dev/) workspace (`melos.yaml`).

```bash
# Rust
cargo build --workspace
cargo test --workspace

# Flutter (per app)
cd apps/music && flutter run -d macos      # or windows / linux / a device

# Melos (dev orchestration)
dart pub global activate melos
melos bootstrap
melos run analyze
melos run test
```

When you change the Rust API exposed to Dart, regenerate the bindings:

```bash
cd apps/music && flutter_rust_bridge_codegen generate
```

## Contributing

Contributions welcome under Apache 2.0 — see [CONTRIBUTING.md](CONTRIBUTING.md).
Use [Conventional Commits](https://www.conventionalcommits.org/) with a component scope
(`feat(music): …`, `fix(engine): …`); releases are automated from them.

## Sponsor

If Cymbra is useful to you, consider sponsoring via the **Sponsor** button.

---

Copyright 2026 NEETROF — Apache License 2.0.
