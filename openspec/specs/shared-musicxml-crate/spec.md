# shared-musicxml-crate Specification

## Purpose
TBD - created by archiving change add-score-crawler. Update Purpose after archive.
## Requirements
### Requirement: Standalone MusicXML crate

The system SHALL provide a `musicxml-core` workspace crate holding the pure
MusicXML data model and streaming parser, with NO dependency on
`flutter_rust_bridge`. The crate SHALL expose the parse entry point and the
score data model as its public API and SHALL be usable by any workspace crate
(the app FFI engine, the backend `score` module, and this crawler). This
extraction is SHARED with `add-user-score-upload` and SHALL be delivered once —
whichever change lands first performs it; the other depends on the existing
crate rather than re-extracting.

#### Scenario: Crate parses without FFI
- **WHEN** a non-FFI crate (the crawler) calls the shared parser on MusicXML
  bytes
- **THEN** it receives the structured score document without linking
  `flutter_rust_bridge`

#### Scenario: Extraction is not duplicated across changes
- **WHEN** both this change and `add-user-score-upload` need the shared parser
- **THEN** a single `musicxml-core` crate exists and both depend on it, rather
  than each lifting its own copy

### Requirement: App engine reuses the shared crate

The system SHALL make the app's FFI seam (`apps/music/rust`) depend on
`musicxml-core` and re-export / wrap its parser and model rather than holding a
private copy. The `#[frb]` wrappers SHALL remain in the app crate; the pure logic
SHALL live only in `musicxml-core`.

#### Scenario: No duplicated parser
- **WHEN** the workspace is built
- **THEN** the MusicXML parser exists once (in `musicxml-core`) and the app crate
  references it, with the FFI wrappers as the only app-side addition

### Requirement: App behaviour unchanged

The extraction SHALL NOT change app-observable behaviour: the generated Dart
MusicXML API and the existing `score-notation` tests SHALL remain valid and
green after the refactor.

#### Scenario: Existing score-notation tests pass
- **WHEN** the app test suite runs after extraction
- **THEN** the `score-notation` unit/widget tests and the generated Dart API are
  unchanged and passing

### Requirement: Validation helper

The `musicxml-core` crate SHALL expose a validation helper that answers whether a
byte buffer is well-formed MusicXML (usable both for native-input validation and
for `.mxl` re-parse verification in the conversion pipeline).

#### Scenario: Validator distinguishes MusicXML from arbitrary XML
- **WHEN** the validator is given well-formed XML that is not MusicXML
- **THEN** it reports the buffer as not valid MusicXML

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

