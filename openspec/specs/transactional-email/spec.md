# transactional-email Specification

## Purpose
TBD - created by archiving change template-backend-emails. Update Purpose after archive.
## Requirements
### Requirement: Branded HTML rendering
Every transactional email the backend sends SHALL be rendered as HTML styled with
the shared Cymbra "Sonic Luminescence" design system, so it is visually
consistent with the Cymbra applications. Because these emails originate from the
**shared identity system** (which serves every audience), they SHALL carry the
umbrella **"Cymbra ID"** identity brand and SHALL NOT use any single product's
branding (e.g. the Cymbra Music logo). The rendered HTML SHALL use the brand
palette (navy `#0B1326` surface, `#131B2E` card, violet `#7C3AED` / lilac
`#D2BBFF` accent, `#DAE2FD` body text), the "Cymbra ID" wordmark in the header,
and a footer carrying the NEETROF attribution and legal links.

#### Scenario: Verification email is branded
- **WHEN** the backend renders the account-verification email
- **THEN** the HTML contains the "Cymbra ID" wordmark, the brand accent color
  `#7C3AED`, and the NEETROF footer
- **AND** it does not reference any single product's logo or branding
- **AND** the verification code is displayed in a distinct, high-contrast box

#### Scenario: Password-reset email is branded
- **WHEN** the backend renders the password-reset email
- **THEN** the HTML uses the same brand layout, wordmark, accent color, and footer
  as the verification email
- **AND** the reset code is displayed in the same distinct, high-contrast box

### Requirement: Localized footer legal links
Every transactional email footer SHALL carry Terms-of-Service and Privacy-Policy
links pointing to the `cymbra.app` legal pages, resolved from the email's locale:
French SHALL use the French pages, and every other locale SHALL fall back to the
English pages (consistent with the `legal-links` resolution used by the apps).

#### Scenario: French footer links
- **WHEN** an email is rendered for a French recipient
- **THEN** the footer Terms link is `https://cymbra.app/cgu/` and the Privacy link
  is `https://cymbra.app/confidentialite/`

#### Scenario: Non-French footer links
- **WHEN** an email is rendered for any non-French locale (including Spanish and Italian)
- **THEN** the footer Terms link is `https://cymbra.app/en/terms/` and the Privacy
  link is `https://cymbra.app/en/privacy/`

### Requirement: Email-client-safe output
Rendered HTML SHALL be robust across common email clients. It SHALL use a
table-based layout with inline `style` attributes only (no `<style>`/`<head>`
CSS, no Flexbox/Grid, no external stylesheets or webfonts). The verification/reset
code SHALL NOT be conveyed by an image — it SHALL always be live text. Any logo
image SHALL degrade gracefully (alt text and/or a text wordmark fallback) so that
when images are blocked the brand name and the code remain fully legible.

#### Scenario: Code survives blocked images
- **WHEN** an email is rendered and a client blocks image loading
- **THEN** the code remains fully visible as text
- **AND** the "Cymbra ID" brand name remains visible via alt text or a text wordmark fallback

#### Scenario: Layout uses inline styles
- **WHEN** an email is rendered
- **THEN** all styling is expressed via inline `style` attributes on table-based
  markup, with no reliance on document-level `<style>` blocks

### Requirement: Plain-text alternative
Every transactional email SHALL be sent as a multipart message carrying both the
branded HTML and a plain-text alternative. The plain-text part SHALL contain the
same code and the same essential instructions, so text-only clients and screen
readers deliver a usable message.

#### Scenario: Multipart message carries both parts
- **WHEN** the backend sends a transactional email
- **THEN** the message has an HTML part and a plain-text alternative part
- **AND** the plain-text part contains the same code as the HTML part

### Requirement: Single shared render layer
All transactional emails SHALL be produced by one shared render layer, so that a
given email is byte-identical regardless of which producer sends it (an inline
sender or an enqueued email job). Adding or restyling an email SHALL be done in
this layer, not at individual call sites.

#### Scenario: Job and inline producers match
- **WHEN** the verification email is produced via the enqueued email job
- **AND** the same verification email is produced via the inline resend path
- **THEN** both are rendered by the same layer and produce identical subject,
  HTML, and plain-text content for the same code and locale

### Requirement: Localized content with English fallback
Transactional email subject and body SHALL be localized to the flagship app's
supported locales (English, Spanish, French, Italian), selected from the
recipient's locale. When no locale is provided, or the provided locale is not
supported, the backend SHALL render the email in English. The recipient's locale
SHALL be accepted as an optional, additive field on the relevant requests such
that omitting it does not break existing clients.

#### Scenario: Renders in the requested supported locale
- **WHEN** an email is rendered for a recipient whose locale is French
- **THEN** the subject and body are in French

#### Scenario: Falls back to English for unknown or missing locale
- **WHEN** an email is rendered with no locale, or with an unsupported locale
- **THEN** the subject and body are rendered in English
- **AND** the email is still fully branded and valid

#### Scenario: Optional locale is backwards compatible
- **WHEN** a client sends a request without the locale field
- **THEN** the request succeeds and the email is rendered in English

