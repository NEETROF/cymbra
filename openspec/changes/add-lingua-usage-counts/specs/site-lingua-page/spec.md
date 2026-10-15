## ADDED Requirements

### Requirement: The Lingua leaving page asks one question with fixed answers
The site SHALL publish a Lingua leaving page in French (`/lingua/au-revoir/`), English (`/en/lingua/goodbye/`) and Spanish (`/es/lingua/adios/`) that asks why the reader is leaving Cymbra Lingua and offers six fixed answers — not using it, too many or too few words highlighted, their language missing, the browser slowed down, not understanding how to use it, another reason — and no free-text field.
- Loading the page SHALL send nothing to Cymbra's backend and SHALL load no analytics, beyond the ordinary page request to the site's static host.
- Choosing an answer SHALL send that one answer, with the browser family and version read from the page's parameters (`unknown` when absent or malformed), to the backend's leaving-reason route, and SHALL then thank the reader whatever the outcome, offering the support page and the community link.
- Without JavaScript, the answers SHALL still work: they SHALL be a plain HTML form that posts the tapped answer (browser family and version then `unknown`) to the same route, which sends the reader back to the page's thanks.
- The page SHALL send at most one answer per visit.
- The three paths SHALL be pinned routes, since shipped extensions open them.
- The island SHALL call the route only through a composable behind the site's client seam, with an `Async<T>` state.

#### Scenario: Opening the page
- **WHEN** Firefox opens `https://cymbra.app/lingua/au-revoir/?b=firefox&v=1.10` after Lingua is removed
- **THEN** the page shows the question and the six answers in French, and no request has been sent to Cymbra's backend

#### Scenario: No JavaScript
- **WHEN** the page is opened in a browser that runs no JavaScript and the reader taps « Une autre raison »
- **THEN** the browser posts `other` to the leaving-reason route as a plain form, and lands back on the page showing the thanks

#### Scenario: Answering
- **WHEN** the reader taps « Ça ralentissait mon navigateur »
- **THEN** one request sends `slow` with `firefox` and `1.10`, the answers are disabled, and the reader is thanked

#### Scenario: The backend is unreachable
- **WHEN** the reader taps an answer and the request fails
- **THEN** the reader is thanked all the same, and no technical error is shown

#### Scenario: Opened by hand
- **WHEN** someone opens `https://cymbra.app/en/lingua/goodbye/` with no parameters and taps an answer
- **THEN** the answer is sent with `unknown` as the browser family and the version

#### Scenario: A route a shipped build depends on
- **WHEN** the route check runs on a build that no longer produces `/es/lingua/adios/`
- **THEN** it fails
