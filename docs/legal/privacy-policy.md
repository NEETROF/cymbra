<!--
  DRAFT — review and complete the «...» fields before publishing, and ideally have a
  lawyer review it. Written for Cymbra (GDPR), aligned with the data actually processed
  by the backend. English version of politique-de-confidentialite.md (the French version
  is legally required for French consumers; keep both in sync).
-->

# Privacy Policy — Cymbra

**Last updated: 18/08/2026**

This policy explains what personal data the **Cymbra** application processes, why, on
what legal basis, who it is shared with, how long it is kept, and what your rights are.

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
| Subscription status (plan, source, start/end dates), **opaque identifiers** of the subscription at the purchase channel (Apple, Google, Paddle — via RevenueCat for the App Store and Google Play), beta campaigns joined, access codes used | purchase channel / you | activate the Premium plan on your devices, manage trials and betas |

We do **not** collect precise location data, do **not** sell any data, and do **not**
use third-party advertising or advertising trackers. We **never** receive or store your
card numbers, billing addresses or invoices: they are processed exclusively by the
purchase channel (Apple, Google, Paddle).

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
  their token), or if you subscribe through the App Store / Google Play (they are the
  merchants of record: they charge and invoice; we never see your payment data).
- **RevenueCat, Inc.** (United States) — only if you subscribe through the App Store or
  Google Play: it verifies the purchase with the store and tracks the subscription
  lifecycle (renewal, grace period, cancellation, refund) on our behalf, and provides
  us with aggregated subscription and revenue analytics. It receives an **opaque
  account identifier**, technical information about the app and device, and the
  store's transaction facts (product, price, currency, country, dates) — never your
  name, email, handle or payment details. This transfer outside the EU is framed by
  the European Commission's Standard Contractual Clauses (data processing agreement
  signed with RevenueCat). Its record is deleted when you delete your account.
- **Paddle** (merchant of record, UK/EU) — only if you subscribe from Linux, Windows or
  the website: it charges, invoices and notifies us of the subscription state bound to
  an opaque identifier.

Your data is **hosted in the European Union** (France). Apart from the subscription
verification described above (RevenueCat, United States, under Standard Contractual
Clauses), we do not transfer data outside the EU.

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
