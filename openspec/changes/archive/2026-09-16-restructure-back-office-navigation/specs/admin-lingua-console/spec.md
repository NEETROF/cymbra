## MODIFIED Requirements

### Requirement: Screen reserved for admins of the lingua scope
The back office SHALL show the Lingua section and its "Overview" navigation entry, and serve the `/lingua/overview` route, only to administrators of the `lingua` scope (the route's access rule is `admin` in `lingua`); any other profile SHALL be redirected without a raw error. The former `/lingua` path SHALL redirect to `/lingua/overview`. This route gate is a UX convenience: every RPC stays independently gated server-side.

#### Scenario: Moderator with neither the link nor access
- **WHEN** a non-admin moderator signed in to the console navigates to `/lingua/overview`
- **THEN** the Lingua section is absent from the navigation and the route redirects them away from `/lingua/overview`

#### Scenario: Music-only admin redirected
- **WHEN** an administrator holding `admin` only in the `music` scope navigates to `/lingua/overview`
- **THEN** the route redirects them (they lack the `lingua` scope), exactly as the server would if they called the RPCs directly

### Requirement: Lingua flags through the existing console
Lingua feature flags SHALL be keys declared in the backend registry (`KeyDef`, app `lingua`) and administered by the existing Feature flags console (`/admin/flags`) with its current gating; this change SHALL introduce no new flags interface.

#### Scenario: A lingua key shows up in the flags console
- **WHEN** a flag key is declared in the backend registry with the `lingua` app
- **THEN** it is listed and administrable in the existing `/admin/flags` console, with no new screen or component
