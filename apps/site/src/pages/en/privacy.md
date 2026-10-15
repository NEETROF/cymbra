---
layout: ../../layouts/Legal.astro
title: Privacy Policy
lang: en
alternates:
  fr: /confidentialite
  en: /en/privacy
  es: /es/privacidad
updated: 15/10/2026
---

This policy explains what personal data the **Cymbra services** (published by
**NEETROF**) process, why, on what legal basis, who it is shared with, how long it is
kept, and what your rights are. It covers the **Cymbra account**, shared across the
Cymbra services; processing specific to a product is set out in an **annex** (see
*Annex A — Cymbra Music* and *Annex B — Cymbra Lingua*).

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

**Figures published in the Cymbra community.** We publish **aggregate** activity figures,
computed from the use of the services, in the public channels of the Cymbra community (our
Discord server): for example the number of sessions played, the average accuracy, the
scores rated and the most played pieces in Cymbra Music, the words read, learned and
reviewed in Cymbra Lingua, or the share of each sign-in method among new accounts. These
figures carry **no name, handle, identifier or number of people**. While the community is
small, a figure may nevertheless reflect the activity of very few people.

## 3. Legal bases (GDPR art. 6)

- **Performance of a contract**: creating and managing your account, providing the app.
- **Legitimate interest**: security, fraud/abuse prevention, rate limiting, correct
  operation, publishing aggregate activity figures in the Cymbra community channels (see
  §2).
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
- **Google** (Gemini API) and **Anthropic, PBC** (Claude API, United States) — only if you
  use Cymbra Lingua's **Online translation** (*Annex B*): they receive the sentence to
  translate, without your account or email, translate it and do not keep it. This
  transfer outside the EU is framed by the European Commission's Standard Contractual
  Clauses in their data processing terms.

Your data is **hosted in the European Union** (France). Apart from the subscription
verification and Online translation described above (RevenueCat, Google and Anthropic,
under Standard Contractual Clauses), we do not transfer data outside the EU.

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
  personal data. Your account is shared by every Cymbra app, so deleting it applies to
  each of them (Cymbra Music, Cymbra Lingua).
- **Cymbra Lingua**: you can also erase **only** your Lingua data, without deleting your
  account (see *Annex B*).
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

---

## Annex B — Cymbra Lingua

**Cymbra Lingua** (browser extension and Safari app) highlights, in the pages you read,
the words you do not know yet in the languages you study. In addition to §2:

**What stays on your device.** Page text is analysed **on your device** and is never sent,
except the sentence you send yourself with **Online translation** and the translation of a word
you choose to keep on its card (below).
Also kept on the device: the detail of your reading (which words you met, on which pages),
the **address of the page** where you add a word to your deck and, when you are not signed
in, all of your Lingua data. Also kept on the device, neither synchronised nor written in the
backup file, is what the request for a rating remembers: the number of review sessions you
finished, the number of days on which you finished one, the date of the last one, the number of
times the request was shown to you and your answer — and, in the Cymbra Lingua app, the number of
sessions finished in the app and the version of the app for which a rating was requested; the
store page opens only if you choose to rate Cymbra Lingua.

**Your languages.** Your language settings stay on the device: the languages you study and
your native language are not stored as such. Each synced status, level, card and day of
statistics is stored under the studied language it belongs to, and your native language
reaches Cymbra as the language of each card's translation and with each day's statistics;
Lingua's interface language, or your browser's, is passed to your Cymbra account as the
account's language (see below).

**Extended translation.** This setting, off by default and specific to each device (it is not
synced), translates on your device the sentence your selection is in. When you turn it on, the
extension downloads **once** the model or models of your language pairs — one model (about
26 MB) when the translation is direct, two (about 52 MB) when it goes through English —, the
Firefox Translations models published by Mozilla under the MPL 2.0, from `models.cymbra.app`;
a pair added later downloads its models only when you ask. Translation then happens **on your
device**, and page text is still never sent. These downloads carry no page text, no Lingua
data, no account token and no installation identifier, and Cymbra does not associate them
with an account or an installation. As with any web page, the host of these files
(Cloudflare) sees the IP address of the device asking for them. Turning the setting off
deletes the models from the device. The Cymbra Lingua app keeps its own copy of the models: it
downloads it, from the same address and on the same terms, only once you accept it in the app,
while the setting is on in the Safari extension of the same device, and deletes it when the
setting is turned off.

**Online translation.** This setting, off by default and specific to each device (it is not
synced), is offered only to accounts that have access to it. When it is on, an « Online
translation » button appears on the translation of a selection of several words, and on a card
under review. **Nothing is sent until you press that button.** When you press it, the sentence
your selection is in — or the reviewed card's sentence, its word located by position —, the
selection's position and the language pair are sent to Cymbra, with your account token, then
by Cymbra to **Google** (Gemini API) to be translated, or to **Anthropic** (Claude API) when
Google does not answer. Neither Cymbra nor these providers keep the sentence or its
translation, and these providers do not use it to train their models. To limit use and know
its cost, Cymbra counts, for your account, the day's online translations (a counter erased
after a day) and keeps daily totals per provider and language pair, with no account and no
text. In the Cymbra Lingua app, Online translation asks you to sign in **inside the app** (with
Apple, Google or your email and password): the app then opens its own session of your account, separate from the
extension's, which you can end from the app.

**A kept translation.** A machine translation (on the device or online) is kept only if, in
review, you choose to keep the translation of the word as its card's translation: only that
translated word is kept, on the card, and synced with it; the translated sentence never is. A
translation you do not keep is stored nowhere.

**Read aloud.** Read aloud uses a voice of the language being read, installed on your device:
the text read does not leave it. If no voice of that language is installed there, you can turn
on your browser's online voices as a fallback (a setting off by default, specific to each
device): the text read is then sent by **your browser** to the provider of those voices
(Google, for Chrome), under that provider's terms. Cymbra does not receive this text. As soon
as a voice of that language is installed on the device, it is used instead.

**Anonymous usage counts.** To know whether Lingua actually helps the people who install it,
the extension may send Cymbra six counts, each once per installation: Lingua is **installed**;
it is **set up** (a first level chosen); a **first page is analysed** in a language you study;
you use it again **the day after** installing it; again **between day 7 and day 13**; again
**between day 30 and day 59**. Each report carries three things only: the **week** of the installation, the
**browser family** (Chromium, Firefox or Safari) and the extension's **version** (for example
1.10).

- **No identifier of any kind** — no installation or device identifier, no account, even when
  you are signed in — and **nothing of what you read**: no address, no site, no page text, no
  word, no studied language, no level.
- **No IP address kept.** As an exception to the technical logs of §2 and §5, Cymbra keeps the
  network address of these reports neither in its database, nor in its logs, nor in the logs of
  the server that passes requests on to it (its "reverse proxy"). The address serves only to
  refuse floods of reports, in memory only, in a scrambled form forgotten within the hour.
- **Sent the week after.** Each count leaves your device only from the Monday after the week it
  happened in, at the extension's next start or use, so the moment it reaches Cymbra does not tell
  the day. Should the extension be stopped just as Cymbra receives a report, that report is sent
  again later and counted twice: the totals can only be slightly too high, never tied to you.
- **On your device**, the day of the installation is kept to tell when you come back — until the
  end of the week that follows your 59th day, **73 days at most** — then forgotten; only its week
  leaves the device. At Cymbra, the counts are weekly totals, with no trace of each report,
  **deleted after 25 months**.
- **Installations from before the counts.** An installation made before the version of Lingua
  that introduced the counts is never counted, never asked, and opens no leaving page.
- **How each browser asks.** Chrome and the other Chromium browsers (Edge…): nothing is sent
  until you press « Yes, count ». Firefox: Firefox's own consent to « Technical and interaction
  data », offered when you install the extension, decides (Firefox 140 or later on desktop, 142
  or later on Android; older versions count nothing). Safari: on by default, once the sentence
  that presents them has been shown.
- **Turning them off.** The « Anonymous usage counts » switch, in Settings → Data, turns them off
  at any time, in every browser (on Firefox, it changes that same consent). Turning them off
  forgets what had not been sent yet.
- **Legal basis**: on Chrome and the other Chromium browsers, your **consent**; elsewhere,
  Cymbra's **legitimate interest** in measuring Lingua's audience. The switch is how you object,
  or withdraw your consent, at any time.

**The leaving page.** On Chrome and Firefox, while the counts are on, removing Lingua opens a
cymbra.app page that asks one question — why you are leaving — with six fixed answers and no
free text. Opening it is an ordinary visit to the site: as for any of its pages, its host
(Cloudflare) receives your device's IP address and the page's address, with its two parameters
(the browser family and the version); Cymbra adds nothing to that visit and receives nothing
from it. Only the answer you press is sent to Cymbra and counted, in the same anonymous way: the
week it arrives, the browser family and the extension's version, the only things the page's
address carries (without JavaScript, the page sends neither, and they are counted as unknown).
With the counts off, no page opens.

**What is synced when you are signed in** to your Cymbra account:

| Data | Source | Purpose |
|---|---|---|
| Each word's status (known, learning, ignored) and your declared level, for each language you study | you | find your progress on all your devices |
| Review deck, for each language you study: the word, the sentence you found it in, its translation — the dictionary's, or the machine translation of the word that you chose to keep, never a translated sentence —, the language of that translation, and its review state — **without** the page address | you | review on all your devices |
| Daily statistics, per day and language you study (words learned, reviews, number of words read and, among them, of new words), with your native language that day (the one the translations are in) | computed on the device | show your statistics; **aggregated** usage figures to run the service and, with no name and no number of people, published in the Cymbra community channels (see §2) |
| Random installation identifier | generated by the extension, and by the Cymbra Lingua app for itself | tell two devices apart during sync |

The extension syncs this data. If you **also sign in inside the Cymbra Lingua app**, with the
account Safari's extension uses, the app itself syncs the statuses and cards of the reviews done
in the app, under its own session and its own installation identifier, with the same content (no
page address); the daily statistics still go through the extension. Nothing more is collected.

- **Legal basis**: performance of a contract (providing sync); legitimate interest for
  publishing aggregate figures (see §2).
- **Retention**: for as long as your account exists, or until you erase your Lingua data.
- **Erase your Lingua data without deleting your account**: in the extension, on the account
  page, the option that erases your Lingua data. The erasure covers the server and all your
  devices (each one clears itself at its next sync); your Cymbra account and your other
  Cymbra apps are not affected.
- **Deleting your account** (§7) also erases all of your Lingua data.
- This data is **hosted in the European Union** (France), like the rest of your data.

**Your Cymbra account.** When you create your account, ask for a code or a new password, or add
a password from Lingua, the extension tells Cymbra a language — Lingua's interface language or
your browser's — which your account keeps as its language: Cymbra writes to you in it
(verification, password reset), and your other Cymbra apps may adopt it.

**Chrome Web Store User Data Policy.** The use of information received from Google APIs will
adhere to the Chrome Web Store User Data Policy, including the Limited Use requirements. For
Cymbra Lingua, this information is: the text of the pages you read, analysed on your device;
your Cymbra account (email address, sign-in identifier, sessions, account language); what is
synced when you are signed in (word statuses and levels, review deck, daily statistics,
installation identifier); and the sentence sent with **Online translation**, only when you
press its button. Cymbra uses this data only for the purposes set out in this policy, shares it
only with the processors named in §4 and for those purposes, does not sell it and does not use
it for advertising. Cymbra's team sees your Lingua data only as aggregates, with no account,
except to help you at your request, for security, or to comply with the law.
