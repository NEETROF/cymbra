## ADDED Requirements

### Requirement: Crate builds for a WebAssembly target

The `musicxml-core` crate SHALL remain buildable for the
`wasm32-unknown-unknown` target so a browser front-end can reuse its parser and
layout engine through a wasm wrapper. No dependency or code that is incompatible
with that target (native FFI, threads, audio, blocking IO, or a crate that fails
to compile for `wasm32`) SHALL be introduced into `musicxml-core`. This asserts
the crate's existing "pure, no-IO/FFI" character as a checked build contract, so a
future regression that breaks the wasm build is caught rather than shipped.

#### Scenario: Core crate compiles to wasm

- **WHEN** the workspace is built for `wasm32-unknown-unknown` targeting the crate
  (directly or via its wasm wrapper)
- **THEN** `musicxml-core` compiles successfully, exposing its parse and layout
  entry points, without pulling any native-only dependency

#### Scenario: A wasm-incompatible dependency is rejected

- **WHEN** a change adds to `musicxml-core` a dependency or code path that does not
  compile for `wasm32-unknown-unknown`
- **THEN** the wasm build fails, flagging the regression before it can break the
  web notation renderer
