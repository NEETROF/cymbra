# localise-cymbra-id-email-legal-links-italian — an Italian e-mail links the Italian legal pages

## Why

Change 43 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. Cymbra ID writes its e-mails (verification code, password reset) in the account's language,
Italian included (`SupportedLocale::It`). Their footer links the Terms and the Privacy Policy, resolved
from that language (`localise-cymbra-id-email-legal-links`): French and Spanish link their own pages,
and every other locale the English ones, Italian included, because the site had no Italian page. Change
41 publishes `/it/termini/` and `/it/privacy/`. I10, settled on 2026-10-22, has an Italian e-mail
follow Italian, as French, English and Spanish follow theirs.

## What Changes

- **`legal_links(SupportedLocale::It)`** (design D1) returns `https://cymbra.app/it/termini/` and
  `https://cymbra.app/it/privacy/`; English keeps the English pages.
- **Deployed after the site** (D2): a sent e-mail keeps its links forever, so the backend ships only
  once change 41's pages are deployed. Change 41 pins both paths, naming this footer as their consumer.
- **Not here** (D3):
  - the locale the extension sends Cymbra ID, which follows the interface language once Italian is
    one, and its deletion link, which change 45 moves to the Italian page (D3);
  - Music's in-app legal links (a Music change, as for Spanish);
  - the privacy policy's Annex B, which names English only as the pivot and stays true; its Italian
    is translated with `/it/privacy/` (change 41).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `transactional-email`: MODIFIED *Localized footer legal links* (held by
  `localise-cymbra-id-email-legal-links`): Italian uses the Italian pages, with a scenario of its own.

## Impact

- **Products.** Cymbra ID's e-mails only: `backend/platform/src/email_template/mod.rs` (`legal_links`,
  its doc comment, its tests). No site, extension, app or Music change.
- **Deploy, the owner's (I12):** the implementation merges only once change 41's site is deployed, and the backend deploys after it.
- **Prerequisites**: change 41 deployed, with `/it/termini/` and `/it/privacy/` pinned.
- **Order**: merged after change 41's site is deployed (D2); independent of change 45.
- **Effort, against 1–2 ideal days**: 0.5–1, as the Spanish precedent's (design, *Effort*).
