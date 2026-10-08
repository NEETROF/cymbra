# Design — add-lingua-spanish-audience-listings

## Context

See proposal.md (Why). The listing files: `apps/lingua-extension/STORE-LISTING.md` — constants (name,
homepage, privacy policy, support, category, listing language French, AMO id), the summary (from
`manifest.json`, 112 characters, Apple's bound), the French and English descriptions, single purpose,
permissions, remote code, data usage, test instructions (≤ 1,000 characters); `apps/lingua-apple/
STORE-LISTING.md` — name, subtitle (30), promotional text (170), keywords (100), description (4,000),
« What's New », review notes (English), URLs, screenshots per platform. Nothing is uploaded by CI.
The requirement *The store listings name each studied language* says "one listing per store" and
"French glosses".

## Goals / Non-Goals

**Goals:**
- A complete Spanish listing in each store, for Spanish speakers learning English, drafted for the owner.

**Non-Goals:**
- Uploading listings (`add-lingua-asc-localisations-upload` is optional, outside the programme's counts).
- Another audience's listing.

## Decisions

### D1 — One listing per store, a text per language

Each store keeps its one listing; the dashboards' per-language fields carry the Spanish texts beside
the French. The files gain a « Spanish » section per field, with its count; the listing-language line
says the listing is in each native language a shipped pair is glossed in.

### D2 — What the Spanish texts say

What a reader of Spanish gets: the highlighted words of the language they study, the honest
percentage, the card in Spanish with its grammar and its gloss written by people (M5), review and
statistics, local analysis and privacy, books, the account; extended translation as M15 settles it
for en-es, with its download size; the coverage figures on the site. The French texts are
unchanged.

### D3 — The App Store locales

es-ES and es-MX, as M16 recommends; the owner settles M16 on this pull request. The review notes stay
English and say which interface language each locale's reviewer sees and how to choose it.

### D4 — Screenshots

A list per platform and locale (iPhone 6.9", iPad 13", Mac; the extension's store images), each
naming the page and the state to capture, taken by the owner from change 35's build.

## Risks / Trade-offs

- **Counts over a store's limit** → each field carries its count, checked in review.
- **A listing that promises what is not shipped** → the texts are written for change 35's
  packages and pasted with them.

## Migration Plan

Pasted by the owner with change 35's release.
