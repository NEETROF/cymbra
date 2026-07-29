# web-notation-render Specification

## Purpose
TBD - created by archiving change add-wasm-notation-preview. Update Purpose after archive.
## Requirements
### Requirement: WebAssembly notation render entry point

The system SHALL provide a WebAssembly module, compiled from the shared notation
engine (`cymbra-musicxml-core`), that exposes a minimal **read-only** entry point
taking raw score bytes and an available render width and returning the laid-out
notation geometry (the parsed score document and its per-line system layout) in a
form a browser can consume. The module SHALL perform no IO, no network access, and
no FFI to native libraries, and SHALL NOT expose any operation that mutates a
score. The geometry SHALL be produced by the **same** layout logic the app uses
(`parse` then `layout_systems`), so the browser and the app derive notation from
one source of truth.

#### Scenario: Bytes are laid out into notation geometry

- **WHEN** the module is called with valid MusicXML (or `.mxl`) bytes and an
  available width
- **THEN** it returns the notation geometry (document + laid-out systems with
  measure widths and note positions) computed by the shared engine

#### Scenario: Layout matches the app's engine

- **WHEN** the same score bytes and width are laid out by the wasm module and by the
  app's `layout_systems`
- **THEN** the resulting system breaks and measure geometry are equivalent, because
  both call the shared core

#### Scenario: Malformed input is reported, not crashed

- **WHEN** the module is called with bytes that are not well-formed MusicXML
- **THEN** it returns a typed error to the caller and does not panic or produce
  partial geometry

#### Scenario: The module is read-only

- **WHEN** any consumer uses the module
- **THEN** the only capability exposed is bytes-to-geometry; there is no operation
  that edits, persists, or otherwise mutates a score

### Requirement: Read-only notation rendered in the browser

The system SHALL render the notation geometry produced by the wasm module into a
**read-only** on-screen view in the browser, drawing staves, clefs, note heads,
stems, beams, accidentals, dots, and rests with SMuFL glyphs, faithfully to how the
app draws the same geometry. The rendered view MUST NOT offer any affordance to edit
the score. The SMuFL font SHALL be served same-origin (no external font fetch), and
the module SHALL be instantiable under the application's content-security policy.

#### Scenario: Notation is drawn from the geometry

- **WHEN** a score's geometry is available
- **THEN** its notation is drawn on screen (staves, clefs, notes, stems, beams,
  accidentals, rests) rather than shown as a bytes/placeholder message

#### Scenario: Rendering is faithful to the app

- **WHEN** a score is rendered in the browser and in the app
- **THEN** the browser view reflects the same notation content and layout the app
  shows for that score, so a reviewer judges it as users will see it

#### Scenario: The rendered view is read-only

- **WHEN** a user views a rendered score
- **THEN** no control edits, reorders, or alters the notation; the view only displays

### Requirement: Renderer is lazy-loaded and isolated behind a fallback-friendly seam

The wasm module and its browser painter SHALL be loaded **on demand** (not part of
the application's initial bundle) and instantiated once, so pages that do not preview
notation pay no download or startup cost for it. The renderer SHALL be encapsulated
behind a single isolated seam so that it can be swapped for an alternative
(pure-JavaScript) renderer, or degrade gracefully, without changing the surrounding
UI. A failure to load, instantiate, or render SHALL degrade to a non-fatal state
(an informative placeholder), never crash the surrounding view.

#### Scenario: Not loaded until a preview is requested

- **WHEN** the application loads and no score preview is requested
- **THEN** the wasm module is not downloaded or instantiated

#### Scenario: Loaded on first preview

- **WHEN** a user first requests a score preview
- **THEN** the module is fetched and instantiated once and reused for subsequent
  previews

#### Scenario: Load or render failure degrades gracefully

- **WHEN** the module fails to load, instantiate, or render a score
- **THEN** the view shows a non-fatal placeholder state and the rest of the page
  keeps working

#### Scenario: Renderer can be swapped at one seam

- **WHEN** the wasm renderer is replaced by an alternative renderer
- **THEN** only the isolated renderer seam changes; the components that display the
  preview are unaffected

