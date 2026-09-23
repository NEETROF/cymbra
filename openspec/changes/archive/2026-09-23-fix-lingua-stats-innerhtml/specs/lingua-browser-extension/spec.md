## ADDED Requirements

### Requirement: Stats view renders without dynamic innerHTML

The stats/review view (shared by the Chromium side panel and the Firefox/Safari in-page drawer) SHALL build its markup with DOM construction APIs (`createElement`, `createElementNS`, `append`, `textContent`) rather than assigning a dynamically-built string to `innerHTML`. This applies whether or not the interpolated data is user-controlled: the rule is about the rendering mechanism, not about proving any particular value is safe to interpolate.

#### Scenario: Rendering the vocabulary estimate, ladder, seed control, and daily cards

- **WHEN** the stats view mounts or re-renders any of its sections (vocabulary estimate, CEFR ladder, "Renforcer un niveau" control, per-metric daily-count cards)
- **THEN** the section's markup is constructed via DOM APIs, with no assignment of a dynamically-built string to an element's `innerHTML`

#### Scenario: A store review adds new stats markup

- **WHEN** a future change adds a new section or control to the stats view
- **THEN** its markup is built the same way — DOM construction, not `innerHTML` — consistent with the rest of the view
