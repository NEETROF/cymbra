---
layout: ../../layouts/Legal.astro
title: Privacy Policy
lang: en
updated: 30/07/2026
---

This policy explains what personal data the **Cymbra services** (published by
**NEETROF**) process, why, on what legal basis, who it is shared with, how long it is
kept, and what your rights are. It covers the **Cymbra account**, shared across the
Cymbra services; processing specific to a product is set out in an **annex** (see
*Annex A — Cymbra Music*).

## 1. Data controller

**NEETROF — SASU, SIREN 948723887**, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCE.
Contact: **gfortin@neetrof.fr**.

## 2. Data we process

We apply **data minimisation**: we only collect what is necessary to operate your
account and the application.

| Data | Source | Purpose |
|---|---|---|
| Email address | you (email sign-up) or your provider (Google/Apple) | account identifier, verification, password reset |
| Password (**argon2** hash, never in clear text) | you (email sign-up) | authentication |
| External sign-in identifier (Google/Apple "sub") | Google / Apple | "Sign in with Google/Apple" |
| Handle and display name | you | identification within the app |
| App preferences | you | remember your settings |
| Session tokens (refresh tokens) | generated at login | keep you signed in |
| Technical logs (IP address, timestamps, errors) | server | security, abuse prevention, correct operation |

We do **not** collect precise location data, do **not** sell any data, and do **not**
use third-party advertising or advertising trackers.

## 3. Legal bases (GDPR art. 6)

- **Performance of a contract**: creating and managing your account, providing the app.
- **Legitimate interest**: security, fraud/abuse prevention, rate limiting, correct
  operation.
- **Consent**: signing in with Google/Apple (you choose this method).

## 4. Processors and third parties

We share only what is necessary with providers acting on our behalf:

- **OVHcloud** (France, EU) — hosting of the server and backups.
- **Brevo** (EU) — sending transactional emails (verification, password reset).
- **Google** / **Apple** — only if you use their sign-in (verifying your identity via
  their token).

Your data is **hosted in the European Union** (France). We do not transfer data outside
the EU.

## 5. Retention

- Account data: kept **for as long as your account exists**.
- **Account deletion**: when you delete your account (see §7), your personal data
  (email, password hash, external identity, handle, name, sessions) is **erased**.
- Backups: encrypted backups rotate over a sliding window (14 days) then are
  overwritten; deleted data therefore disappears at the latest when that window
  expires.
- Technical logs: 7 days.

## 6. Security

Encryption in transit (**TLS**), passwords stored as an **argon2** hash (never in clear
text), **encrypted** backups stored off the server, restricted server access. As no
measure is infallible, we cannot guarantee absolute security.

## 7. Your rights (GDPR)

You have the rights of **access**, **rectification**, **erasure**, **restriction**,
**objection** and **portability**.

- **Erasure (right to be forgotten)**: you can **delete your account directly in the
  app** (Settings → Delete my account). Deletion is irreversible and erases your
  personal data.
- For any other request, write to **privacy@cymbra.app**. You may also lodge a
  complaint with the French supervisory authority, the **CNIL** (www.cnil.fr), or your
  local data protection authority.

## 8. Minors

Cymbra is not intended for children under 12; we do not knowingly collect their data.

## 9. Changes

We may update this policy; the "Last updated" date above will change accordingly. We
will inform you of any material change.

## 10. Contact

**gfortin@neetrof.fr** — NEETROF, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCE.

---

## Annex A — Cymbra Music

The **Cymbra Music** service lets you upload your own content. For that purpose, in
addition to §2, we process:

| Data | Source | Purpose |
|---|---|---|
| Uploaded files (scores, piano sounds / *soundfonts*) | you | provide playback and practice features |
| Associated metadata (file name, origin attestation, timestamp) | you | management and traceability of your content |

- **Legal basis**: performance of a contract (providing the feature).
- **Retention**: for as long as you keep the content; **deletion** removes the file and
  its record (see Terms of Service, *Annex A — Cymbra Music*).
- These files are **hosted in the European Union** (France), like the rest of your data.
