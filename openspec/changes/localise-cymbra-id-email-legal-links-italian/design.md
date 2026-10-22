# Design — localise-cymbra-id-email-legal-links-italian

## Context

- `backend/platform/src/email_template/mod.rs`'s `legal_links(locale)` maps:
  - `Fr` to `/cgu/` and `/confidentialite/`;
  - `Es` to `/es/terminos/` and `/es/privacidad/`;
  - `En | It` to `/en/terms/` and `/en/privacy/`.

  Its doc comment says Italian « has no site pages ». The layout already declares the e-mail's
  language (`lang="it"` for Italian), and the Italian copy exists.
- Matrix change 32 (`localise-cymbra-id-email-legal-links`) made the rule the e-mails' own:
  - Music's in-app links keep theirs (the `legal-links` spec: French its pages, every other language
    the English ones);
  - the two Spanish pages were pinned on the site;
  - the site was ordered before the backend (its task 2.2, the owner's).
- Change 41 (`add-site-italian-locale`) publishes `/it/termini/` and `/it/privacy/` and pins them
  ahead of their consumers, this footer among them.
- I10 (settled 2026-10-22): account e-mails and the deletion link follow the Italian interface
  language, like French, English and Spanish.

## Goals / Non-Goals

**Goals:**
- An e-mail written in Italian links the Italian Terms and Privacy Policy.

**Non-Goals:**
- Which locale an account has. Cymbra ID writes in the account's language; the extension's choice of
  locale follows its interface language and becomes Italian with change 45, whose requirement change
  carries it.
- Music's in-app legal links, which keep the English pages for Italian, as they did for Spanish.

## Decisions

### D1 — Italian links its own pages

`legal_links`'s `En | It` arm splits: `It` returns `("https://cymbra.app/it/termini/",
"https://cymbra.app/it/privacy/")`, and `En` keeps the English pages. The doc comment, which says Italian
« has no site pages », names the four languages and keeps saying that every path is pinned on the site.
The test `italian_uses_english_legal_links` (with its comment « No Italian site pages ») becomes
`italian_uses_italian_legal_links`: both `/it/` addresses in both parts of the e-mail, and no
`https://cymbra.app/en/`, as `spanish_uses_spanish_legal_links` asserts. `english_uses_english_legal_links`
and the unsupported locale's test (`de`) are unchanged. Nothing else maps Italian to English: the
layout only interpolates the two addresses and `lang`.

The paths are the ones change 41 pins; if its review renamed one before pinning, this change follows
before it is implemented.

### D2 — The site first, then the backend

A sent e-mail keeps its links forever, so this backend change ships only once change 41's site is
deployed with both pages. Change 41's D3 pins them (`pinnedBy` « Cymbra ID e-mail footer, it (change
43) », source `backend/platform/src/email_template/mod.rs`; `/it/privacy/` also for change 45's privacy
link), so `check:routes` refuses a site build that drops them from then on.

The two changes need opposite orders, and the backend ships as one image (a `backend-v*` tag, or
`latest` for main's head). Change 41 deploys its backend (`thanks_path`) before its site; this change
needs the site before its backend. If this change's implementation merged before change 41's backend
were released, the deploy change 41 needs would carry the Italian links before the pages exist, and
Italian e-mails go out today (Music's Italian accounts). So this change's implementation pull request
merges only once change 41's site is deployed, the owner's gate (task 2.2), as
`send-lingua-browser-locale-on-account-emails` gates its own merge.

### D3 — Not here

- **The extension's locale for Cymbra ID.** Once the reader has chosen their native language,
  `accountLocale` keeps the interface language, unless the browser's language is one Cymbra speaks and
  the extension does not (`INTERFACE_LANGUAGES`), when it sends the browser's. So an Italian interface
  sends `it` as soon as change 45 adds Italian to `INTERFACE_LANGUAGES`, with no code here; the Safari
  host app's `appAccountLocale` follows the shared interface language the same way. What changes is the
  « an Italian browser » case: once `it` is an interface language, a French interface with an Italian
  browser sends `fr`, and the clause « the browser's language instead when … the extension does not »
  is left with no language. *The account's e-mails follow the interface language* (held by
  `send-lingua-browser-locale-on-account-emails`) names « `fr`, `en` or `es` » and has a scenario *An
  Italian browser*; `test/account-locale.spec.ts` tests it, and `src/account/locale.ts`'s comment names
  `it`. Change 45 modifies the requirement, the scenario, the tests and the comment. Programme row 45
  says so.
- **The deletion link.** Its page is *Cymbra account deletion is reachable from Lingua* (`lingua-privacy`,
  held by `add-site-spanish-locale`), and `deleteAccountUrl` handles French alone: Spanish still gets the
  English page, change 29's task 3.2 left undone. Change 45 adds Italian's arm and Spanish's with it,
  modifies that requirement (`archiveAfter: add-site-spanish-locale`), and updates `/en/delete-account/`'s
  `pinnedBy` (« every interface language but French and Spanish »).
- **Music's in-app legal links** (`apps/music/lib/services/legal_links.dart`) keep the English pages for
  Italian, as they kept them for Spanish: a Music change, if wanted.
- **The privacy policy's Annex B** names no studied or native language; it names English only as the
  pivot (« deux (environ 52 Mo) quand elle passe par l'anglais »). Every Italian route not to or from
  English goes through English, two models at most (the programme's *Translation*; about 26 MB each,
  change 2), and the account's language is « celle de l'interface de Lingua ou celle de votre
  navigateur », true under I10. So it stays true with Italian. Its Italian is translated with
  `/it/privacy/` (change 41), so row 43's « the privacy annex in Italian » is done there.

## Risks / Trade-offs

- **An e-mail sent before the pages exist** → D2's order; the pins make the pages permanent once
  deployed.

## Migration Plan

The owner deploys the backend after change 41's site. Rollback is a revert: the pinned pages stay, and
an e-mail already sent keeps working.

## Effort

0.5–1 ideal day, as the Spanish precedent's: the arm, its tests and comment 0.25; the gates and the
deploy note 0.25–0.5.
